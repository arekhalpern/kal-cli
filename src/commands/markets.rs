use std::collections::BTreeMap;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{ArgAction, Args, Subcommand, ValueEnum};
use serde_json::Value;

use crate::{
    client::KalshiClient,
    output::{get_i64, print_value, render_markets_table, render_markets_top_table},
    AppContext,
};

#[derive(Debug, Clone, ValueEnum)]
enum MarketStatus {
    Open,
    Closed,
    Settled,
    Unopened,
}

#[derive(Debug, Clone, Args)]
pub struct MarketsCmd {
    #[command(subcommand)]
    command: MarketsSubcmd,
}

#[derive(Debug, Clone, Subcommand)]
enum MarketsSubcmd {
    List {
        #[arg(long)]
        status: Option<MarketStatus>,
        #[arg(long, action = ArgAction::Set)]
        active: Option<bool>,
        #[arg(long = "event")]
        event_ticker: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: usize,
        #[arg(long, default_value_t = false)]
        compact: bool,
    },
    Get {
        ticker: String,
    },
    Search {
        query: String,
        #[arg(long, default_value_t = false)]
        compact: bool,
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
    Orderbook {
        ticker: String,
        #[arg(long)]
        depth: Option<usize>,
    },
}

pub async fn run(ctx: &AppContext, cmd: MarketsCmd) -> anyhow::Result<()> {
    let client = KalshiClient::new(ctx.runtime.clone())?;

    match cmd.command {
        MarketsSubcmd::List {
            status,
            active,
            event_ticker,
            limit,
            compact,
        } => {
            let mut q = BTreeMap::new();
            q.insert("limit".to_string(), limit.to_string());
            if let Some(is_active) = active {
                if is_active {
                    q.insert("status".to_string(), "open".to_string());
                } else {
                    q.insert("status".to_string(), "closed".to_string());
                }
            } else if let Some(s) = status {
                q.insert("status".to_string(), format_status(&s).to_string());
            }
            if let Some(e) = event_ticker {
                q.insert("event_ticker".to_string(), e);
            }

            let data = client.get_public("/markets", Some(q)).await?;
            let mut markets = data
                .get("markets")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            sort_markets(&mut markets);
            enrich_event_market_counts(&client, &mut markets).await?;

            render_markets_table(ctx.output_mode, &markets, compact)
        }
        MarketsSubcmd::Get { ticker } => {
            let data = client.get_public(&format!("/markets/{ticker}"), None).await?;
            print_value(ctx.output_mode, &data)
        }
        MarketsSubcmd::Search {
            query,
            compact,
        } => {
            let mut q = BTreeMap::new();
            q.insert("limit".to_string(), "200".to_string());

            let data = client.get_public("/markets", Some(q)).await?;
            let mut markets = data
                .get("markets")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();

            let q_lower = query.to_lowercase();
            markets.retain(|m| {
                let ticker = m
                    .get("ticker")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_lowercase();
                let title = m
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_lowercase();
                ticker.contains(&q_lower) || title.contains(&q_lower)
            });

            sort_markets(&mut markets);
            enrich_event_market_counts(&client, &mut markets).await?;
            render_markets_table(ctx.output_mode, &markets, compact)
        }
        MarketsSubcmd::Top {
            limit,
            days,
            min_open_interest,
            min_total_volume,
            active,
            include_mve,
            universe,
        } => {
            let target_universe = universe.clamp(1, 10_000);
            let mut q = BTreeMap::new();
            q.insert(
                "status".to_string(),
                if active { "open" } else { "closed" }.to_string(),
            );
            q.insert("max_close_ts".to_string(), upcoming_max_close_ts(days).to_string());
            if !include_mve {
                q.insert("mve_filter".to_string(), "exclude".to_string());
            }

            let mut markets = fetch_markets_universe(&client, q, target_universe).await?;
            markets.retain(|m| {
                let oi = get_i64(m, "open_interest").unwrap_or(0);
                let vol = get_i64(m, "volume").unwrap_or(0);
                oi >= min_open_interest && vol >= min_total_volume
            });
            sort_markets_by_oi_volume(&mut markets);
            markets.truncate(limit);
            enrich_event_market_counts(&client, &mut markets).await?;

            render_markets_top_table(ctx.output_mode, &markets)
        }
        MarketsSubcmd::Orderbook { ticker, depth } => {
            let mut q = BTreeMap::new();
            if let Some(d) = depth {
                q.insert("depth".to_string(), d.to_string());
            }
            let data = client
                .get_public(&format!("/markets/{ticker}/orderbook"), if q.is_empty() { None } else { Some(q) })
                .await?;
            print_value(ctx.output_mode, &data)
        }
    }
}

fn sort_markets(markets: &mut [Value]) {
    markets.sort_by(|a, b| {
        let av = get_i64(a, "volume_24h").or_else(|| get_i64(a, "volume")).unwrap_or(0);
        let bv = get_i64(b, "volume_24h").or_else(|| get_i64(b, "volume")).unwrap_or(0);
        bv.cmp(&av)
    });
}

fn sort_markets_by_oi_volume(markets: &mut [Value]) {
    markets.sort_by(|a, b| {
        let a_oi = get_i64(a, "open_interest").unwrap_or(0);
        let b_oi = get_i64(b, "open_interest").unwrap_or(0);
        let a_vol = get_i64(a, "volume").unwrap_or(0);
        let b_vol = get_i64(b, "volume").unwrap_or(0);
        let a_vol24 = get_i64(a, "volume_24h").unwrap_or(0);
        let b_vol24 = get_i64(b, "volume_24h").unwrap_or(0);

        b_oi
            .cmp(&a_oi)
            .then_with(|| b_vol.cmp(&a_vol))
            .then_with(|| b_vol24.cmp(&a_vol24))
    });
}

fn upcoming_max_close_ts(days: i64) -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let horizon_days = days.max(1);
    now + horizon_days * 86_400
}

async fn fetch_markets_universe(
    client: &KalshiClient,
    base_query: BTreeMap<String, String>,
    target_universe: usize,
) -> anyhow::Result<Vec<Value>> {
    let mut all_markets: Vec<Value> = Vec::with_capacity(target_universe.min(2000));
    let mut cursor: Option<String> = None;
    let page_size = 1000usize.min(target_universe.max(1));

    loop {
        let mut q = base_query.clone();
        q.insert("limit".to_string(), page_size.to_string());
        if let Some(c) = &cursor {
            q.insert("cursor".to_string(), c.clone());
        }

        let data = client.get_public("/markets", Some(q)).await?;
        let page = data
            .get("markets")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        if page.is_empty() {
            break;
        }
        all_markets.extend(page);
        if all_markets.len() >= target_universe {
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

    all_markets.truncate(target_universe);
    Ok(all_markets)
}

async fn enrich_event_market_counts(
    client: &KalshiClient,
    markets: &mut [Value],
) -> anyhow::Result<()> {
    let mut counts: HashMap<String, i64> = HashMap::new();

    for market in markets.iter_mut() {
        let event_ticker = market
            .get("event_ticker")
            .and_then(Value::as_str)
            .map(str::to_string);

        let Some(event_ticker) = event_ticker else {
            continue;
        };

        let count = if let Some(cached) = counts.get(&event_ticker) {
            *cached
        } else {
            let mut q = BTreeMap::new();
            q.insert("with_nested_markets".to_string(), "true".to_string());
            let data = client
                .get_public(&format!("/events/{event_ticker}"), Some(q))
                .await?;
            let cnt = data
                .get("event")
                .and_then(|e| e.get("markets"))
                .and_then(Value::as_array)
                .map(|a| a.len() as i64)
                .or_else(|| data.get("markets").and_then(Value::as_array).map(|a| a.len() as i64))
                .unwrap_or(0);
            counts.insert(event_ticker.clone(), cnt);
            cnt
        };

        if let Some(obj) = market.as_object_mut() {
            obj.insert("event_market_count".to_string(), Value::from(count));
        }
    }

    Ok(())
}

fn format_status(status: &MarketStatus) -> &'static str {
    match status {
        MarketStatus::Open => "open",
        MarketStatus::Closed => "closed",
        MarketStatus::Settled => "settled",
        MarketStatus::Unopened => "unopened",
    }
}
