use std::fmt;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{ArgAction, Args, Subcommand, ValueEnum};
use serde_json::{Map, Value};

use crate::{
    client::KalshiClient,
    output::{extract_array, get_i64, print_value, render_events_table, render_events_top_table, OutputMode},
    query::QueryParams,
    AppContext,
};

#[derive(Debug, Clone, ValueEnum)]
enum EventStatus {
    Open,
    Closed,
    Settled,
}

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Settled => "settled",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Args)]
pub struct EventsCmd {
    #[command(subcommand)]
    command: EventsSubcmd,
}

#[derive(Debug, Clone, Subcommand)]
enum EventsSubcmd {
    List {
        #[arg(long)]
        status: Option<EventStatus>,
        #[arg(long = "series")]
        series_ticker: Option<String>,
        #[arg(long = "with-markets", default_value_t = false)]
        with_markets: bool,
    },
    Get {
        ticker: String,
        #[arg(long = "with-markets", default_value_t = false)]
        with_markets: bool,
    },
    Top {
        #[arg(long, default_value_t = 25)]
        limit: usize,
        #[arg(long, default_value_t = 7)]
        days: i64,
        #[arg(long, default_value_t = 0)]
        min_open_interest: i64,
        #[arg(long, default_value_t = 0)]
        min_total_volume: i64,
        #[arg(long, action = ArgAction::Set, default_value_t = true)]
        active: bool,
        #[arg(long, default_value_t = false)]
        include_mve: bool,
        #[arg(long, default_value_t = 1000)]
        universe: usize,
    },
}

pub async fn run(ctx: &AppContext, cmd: EventsCmd) -> anyhow::Result<()> {
    let client = KalshiClient::new(ctx.runtime.clone())?;

    match cmd.command {
        EventsSubcmd::List {
            status,
            series_ticker,
            with_markets,
        } => {
            // Table mode needs market counts; fetch nested markets automatically.
            let include_markets = with_markets || matches!(ctx.output_mode, OutputMode::Table);
            let status_text = status.as_ref().map(|s| s.to_string());
            let q = QueryParams::new()
                .limit(100)
                .optional("status", status_text.as_deref())
                .optional("series_ticker", series_ticker)
                .optional("with_nested_markets", include_markets.then_some("true"))
                .build_always();

            let data = client.get_public("/events", Some(q)).await?;
            let events = extract_array(&data, "events");
            render_events_table(ctx.output_mode, &events, status_text.as_deref())
        }
        EventsSubcmd::Get { ticker, with_markets } => {
            let q = QueryParams::new()
                .optional("with_nested_markets", with_markets.then_some("true"))
                .build();
            let data = client
                .get_public(&format!("/events/{ticker}"), q)
                .await?;
            print_value(ctx.output_mode, &data)
        }
        EventsSubcmd::Top {
            limit,
            days,
            min_open_interest,
            min_total_volume,
            active,
            include_mve,
            universe,
        } => {
            let target_universe = universe.clamp(1, 10_000);
            let q = QueryParams::new()
                .insert("status", if active { "open" } else { "closed" })
                .insert("with_nested_markets", "true")
                .optional("mve_filter", (!include_mve).then_some("exclude"))
                .build_always();

            let events = fetch_events_universe(&client, q, target_universe).await?;
            let mut rows = aggregate_events(events, days, min_open_interest, min_total_volume);
            sort_top_events(&mut rows);
            rows.truncate(limit);
            let status_fallback = Some(if active { "open" } else { "closed" });

            render_events_top_table(ctx.output_mode, &rows, status_fallback)
        }
    }
}

async fn fetch_events_universe(
    client: &KalshiClient,
    base_query: BTreeMap<String, String>,
    target_universe: usize,
) -> anyhow::Result<Vec<Value>> {
    let mut all_events: Vec<Value> = Vec::with_capacity(target_universe.min(2000));
    let mut cursor: Option<String> = None;
    let page_size = 200usize.min(target_universe.max(1));

    loop {
        let mut q = base_query.clone();
        q.insert("limit".to_string(), page_size.to_string());
        if let Some(c) = &cursor {
            q.insert("cursor".to_string(), c.clone());
        }

        let data = client.get_public("/events", Some(q)).await?;
        let page = extract_array(&data, "events");

        if page.is_empty() {
            break;
        }
        all_events.extend(page);
        if all_events.len() >= target_universe {
            break;
        }

        cursor = data
            .get("cursor")
            .and_then(Value::as_str)
            .map(str::to_string)
            .filter(|c| !c.is_empty());
        if cursor.is_none() {
            break;
        }
    }

    all_events.truncate(target_universe);
    Ok(all_events)
}

fn aggregate_events(
    events: Vec<Value>,
    days: i64,
    min_open_interest: i64,
    min_total_volume: i64,
) -> Vec<Value> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let horizon = now + days.max(1) * 86_400;
    let enforce_horizon = days > 0;

    let mut rows = Vec::with_capacity(events.len());
    for event in events {
        let markets = extract_array(&event, "markets");
        if markets.is_empty() {
            continue;
        }

        let mut total_volume = 0_i64;
        let mut open_interest = 0_i64;
        let mut market_count = 0_i64;
        let mut has_upcoming = false;

        for market in markets {
            total_volume += get_i64(&market, "volume").unwrap_or(0);
            open_interest += get_i64(&market, "open_interest").unwrap_or(0);
            market_count += 1;

            if let Some(close_ts) = market_close_ts(&market) {
                if close_ts <= horizon {
                    has_upcoming = true;
                }
            } else {
                has_upcoming = true;
            }
        }

        if enforce_horizon && !has_upcoming {
            continue;
        }
        if open_interest < min_open_interest || total_volume < min_total_volume {
            continue;
        }

        let mut row = event.as_object().cloned().unwrap_or_else(Map::new);
        row.insert("market_count".to_string(), Value::from(market_count));
        row.insert("total_volume".to_string(), Value::from(total_volume));
        row.insert("open_interest".to_string(), Value::from(open_interest));
        rows.push(Value::Object(row));
    }

    rows
}

fn sort_top_events(rows: &mut [Value]) {
    rows.sort_by(|a, b| {
        let a_oi = get_i64(a, "open_interest").unwrap_or(0);
        let b_oi = get_i64(b, "open_interest").unwrap_or(0);
        let a_vol = get_i64(a, "total_volume").unwrap_or(0);
        let b_vol = get_i64(b, "total_volume").unwrap_or(0);
        let a_cnt = get_i64(a, "market_count").unwrap_or(0);
        let b_cnt = get_i64(b, "market_count").unwrap_or(0);

        b_oi
            .cmp(&a_oi)
            .then_with(|| b_vol.cmp(&a_vol))
            .then_with(|| b_cnt.cmp(&a_cnt))
    });
}

fn market_close_ts(market: &Value) -> Option<i64> {
    for key in ["close_time", "close_ts", "expiration_ts", "expiration_time"] {
        if let Some(v) = market.get(key) {
            if let Some(n) = v.as_i64() {
                return Some(n);
            }
            if let Some(s) = v.as_str() {
                if let Ok(n) = s.parse::<i64>() {
                    return Some(n);
                }
            }
        }
    }
    None
}
