use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{ArgAction, Args, Subcommand, ValueEnum};
use serde_json::Value;

use crate::{
    client::KalshiClient,
    output::{extract_array, get_i64, print_value, render_markets_table, render_markets_top_table},
    query::QueryParams,
    AppContext,
};

#[derive(Debug, Clone, ValueEnum)]
enum MarketStatus {
    Open,
    Closed,
    Settled,
    Unopened,
}

impl fmt::Display for MarketStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Settled => "settled",
            Self::Unopened => "unopened",
        };
        f.write_str(value)
    }
}

const SEARCH_PAGE_SIZE: usize = 1000;
const SEARCH_DEFAULT_LIMIT: usize = 25;
const SEARCH_FALLBACK_DAYS: i64 = 30;

#[derive(Debug, Clone)]
struct FuzzyQuery {
    normalized: String,
    tokens: Vec<String>,
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
        #[arg(long, default_value_t = true)]
        compact: bool,
    },
    Get {
        ticker: String,
    },
    Search {
        query: String,
        #[arg(long, default_value_t = 3)]
        days: i64,
        #[arg(long, default_value_t = SEARCH_DEFAULT_LIMIT)]
        limit: usize,
        #[arg(long, default_value_t = true)]
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
            let status_filter = if let Some(is_active) = active {
                Some(if is_active { "open" } else { "closed" }.to_string())
            } else {
                status.map(|s| s.to_string())
            };
            let q = QueryParams::new()
                .limit(limit)
                .optional("status", status_filter)
                .optional("event_ticker", event_ticker)
                .build_always();

            let data = client.get_public("/markets", Some(q)).await?;
            let mut markets = extract_array(&data, "markets");
            sort_markets(&mut markets);
            enrich_event_market_counts(&client, &mut markets).await?;

            render_markets_table(ctx.output_mode, &markets, compact)
        }
        MarketsSubcmd::Get { ticker } => {
            let data = client
                .get_public(&format!("/markets/{ticker}"), None)
                .await?;
            print_value(ctx.output_mode, &data)
        }
        MarketsSubcmd::Search {
            query,
            days,
            limit,
            compact,
        } => {
            let mut markets = search_open_markets(&client, &query, Some(days)).await?;
            if markets.is_empty() {
                markets = search_open_markets(&client, &query, Some(SEARCH_FALLBACK_DAYS)).await?;
            }
            sort_markets_by_volume(&mut markets);
            markets.truncate(limit);
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
            let q = QueryParams::new()
                .insert("status", if active { "open" } else { "closed" })
                .insert("max_close_ts", upcoming_max_close_ts(days))
                .optional("mve_filter", (!include_mve).then_some("exclude"))
                .build_always();

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
            let q = QueryParams::new().optional("depth", depth).build();
            let data = client
                .get_public(&format!("/markets/{ticker}/orderbook"), q)
                .await?;
            print_value(ctx.output_mode, &data)
        }
    }
}

fn sort_markets(markets: &mut [Value]) {
    markets.sort_by(|a, b| {
        let av = market_volume(a);
        let bv = market_volume(b);
        bv.cmp(&av)
    });
}

fn sort_markets_by_volume(markets: &mut [Value]) {
    markets.sort_by(|a, b| {
        let av = market_volume(a);
        let bv = market_volume(b);
        bv.cmp(&av)
    });
}

fn sort_markets_by_oi_volume(markets: &mut [Value]) {
    markets.sort_by(|a, b| {
        let a_oi = get_i64(a, "open_interest").unwrap_or(0);
        let b_oi = get_i64(b, "open_interest").unwrap_or(0);
        let a_vol = market_volume(a);
        let b_vol = market_volume(b);
        let a_vol24 = get_i64(a, "volume_24h").unwrap_or(0);
        let b_vol24 = get_i64(b, "volume_24h").unwrap_or(0);

        b_oi.cmp(&a_oi)
            .then_with(|| b_vol.cmp(&a_vol))
            .then_with(|| b_vol24.cmp(&a_vol24))
    });
}

fn market_volume(market: &Value) -> i64 {
    get_i64(market, "open_volume")
        .or_else(|| get_i64(market, "volume"))
        .or_else(|| get_i64(market, "volume_24h"))
        .unwrap_or(0)
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
        let page = extract_array(&data, "markets");

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

async fn fetch_open_markets_universe(
    client: &KalshiClient,
    days: Option<i64>,
) -> anyhow::Result<Vec<Value>> {
    let mut all_markets: Vec<Value> = Vec::with_capacity(2000);
    let mut cursor: Option<String> = None;
    let max_close_ts = days.map(|n| upcoming_max_close_ts(n.max(1)));

    loop {
        let q = QueryParams::new()
            .insert("status", "open")
            .optional("max_close_ts", max_close_ts)
            .limit(SEARCH_PAGE_SIZE)
            .optional("cursor", cursor.as_deref())
            .build_always();
        let data = client.get_public("/markets", Some(q)).await?;
        let page = extract_array(&data, "markets");

        if page.is_empty() {
            break;
        }
        all_markets.extend(page);

        cursor = data
            .get("cursor")
            .and_then(Value::as_str)
            .map(str::to_string)
            .filter(|c| !c.is_empty());
        if cursor.is_none() {
            break;
        }
    }

    Ok(all_markets)
}

async fn search_open_markets_in_series(
    client: &KalshiClient,
    series_ticker: &str,
    query: &FuzzyQuery,
) -> anyhow::Result<Vec<Value>> {
    let mut events: Vec<Value> = Vec::with_capacity(256);
    let mut cursor: Option<String> = None;

    loop {
        let q = QueryParams::new()
            .insert("series_ticker", series_ticker)
            .insert("status", "open")
            .insert("with_nested_markets", "true")
            .limit(200)
            .optional("cursor", cursor.as_deref())
            .build_always();
        let data = client.get_public("/events", Some(q)).await?;
        let page = extract_array(&data, "events");
        if page.is_empty() {
            break;
        }
        events.extend(page);

        cursor = data
            .get("cursor")
            .and_then(Value::as_str)
            .map(str::to_string)
            .filter(|c| !c.is_empty());
        if cursor.is_none() {
            break;
        }
    }

    let mut results = Vec::new();

    for event in events {
        let event_match = row_matches(
            &event,
            &["ticker", "event_ticker", "series_ticker", "title", "sub_title"],
            query,
        );
        let markets = extract_array(&event, "markets");
        let event_market_count = markets.len() as i64;

        for mut market in markets {
            let is_active = market
                .get("status")
                .and_then(Value::as_str)
                .map(is_active_market_status)
                .unwrap_or(true);
            if !is_active {
                continue;
            }

            let market_match = row_matches(
                &market,
                &[
                    "ticker",
                    "event_ticker",
                    "title",
                    "yes_sub_title",
                    "no_sub_title",
                    "subtitle",
                ],
                query,
            );

            if !(event_match || market_match) {
                continue;
            }

            if let Some(obj) = market.as_object_mut() {
                obj.insert(
                    "event_market_count".to_string(),
                    Value::from(event_market_count),
                );
            }
            results.push(market);
        }
    }

    Ok(results)
}

async fn search_open_markets(
    client: &KalshiClient,
    query: &str,
    days: Option<i64>,
) -> anyhow::Result<Vec<Value>> {
    let fuzzy = FuzzyQuery::new(query);
    if is_ticker_like_query(query) {
        let series_results =
            search_open_markets_in_series(client, &query.to_ascii_uppercase(), &fuzzy).await?;
        if !series_results.is_empty() {
            return Ok(series_results);
        }
    }
    let mut markets = fetch_open_markets_universe(client, days).await?;

    markets.retain(|market| {
        let is_active = market
            .get("status")
            .and_then(Value::as_str)
            .map(is_active_market_status)
            .unwrap_or(true);
        if !is_active {
            return false;
        }

        row_matches(
            market,
            &[
                "ticker",
                "event_ticker",
                "title",
                "yes_sub_title",
                "no_sub_title",
                "subtitle",
            ],
            &fuzzy,
        )
    });

    Ok(markets)
}

fn row_matches(row: &Value, keys: &[&str], query: &FuzzyQuery) -> bool {
    let combined = keys
        .iter()
        .filter_map(|key| row.get(key).and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join(" ");
    fuzzy_contains(&combined, query)
}

fn is_active_market_status(status: &str) -> bool {
    status.eq_ignore_ascii_case("active") || status.eq_ignore_ascii_case("open")
}

fn is_ticker_like_query(query: &str) -> bool {
    !query.is_empty()
        && !query.chars().any(char::is_whitespace)
        && query
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

impl FuzzyQuery {
    fn new(query: &str) -> Self {
        let normalized = normalize_for_match(query);
        let tokens = normalized
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>();
        Self { normalized, tokens }
    }
}

fn fuzzy_contains(value: &str, query: &FuzzyQuery) -> bool {
    let value_norm = normalize_for_match(value);
    if value_norm.is_empty() || query.normalized.is_empty() {
        return false;
    }
    if value_norm.contains(&query.normalized) {
        return true;
    }

    let value_tokens = value_norm.split_whitespace().collect::<Vec<_>>();
    let matched_flags = query
        .tokens
        .iter()
        .map(|token| token_matches(token, &value_tokens, &value_norm))
        .collect::<Vec<_>>();
    let matched = matched_flags.iter().filter(|matched| **matched).count();

    if query.tokens.len() >= 3 {
        let has_significant = query.tokens.iter().any(|token| !is_location_like_token(token));
        let significant_matched = query
            .tokens
            .iter()
            .zip(matched_flags.iter())
            .any(|(token, matched)| !is_location_like_token(token) && *matched);
        if has_significant && !significant_matched {
            return false;
        }
    }

    matched >= required_token_matches(query.tokens.len())
}

fn token_matches(query_token: &str, value_tokens: &[&str], value_norm: &str) -> bool {
    if query_token.len() <= 2 {
        return value_norm.contains(query_token);
    }

    let mut candidates = vec![query_token.to_string()];
    candidates.extend(token_aliases(query_token).iter().map(|token| token.to_string()));

    candidates.iter().any(|candidate| {
        if candidate.contains(' ') {
            value_norm.contains(candidate)
        } else {
            value_tokens.iter().any(|value_token| {
                value_token.contains(candidate.as_str())
                    || (candidate.len() >= 5
                        && value_token.len() >= 5
                        && edit_distance_at_most(value_token, candidate, 1))
            })
        }
    })
}

fn required_token_matches(token_count: usize) -> usize {
    match token_count {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        _ => token_count - 1,
    }
}

fn is_location_like_token(token: &str) -> bool {
    matches!(
        token,
        "new"
            | "york"
            | "city"
            | "los"
            | "san"
            | "las"
            | "st"
            | "saint"
            | "at"
            | "in"
            | "on"
            | "vs"
            | "the"
    )
}

fn token_aliases(token: &str) -> &'static [&'static str] {
    match token {
        "basketball" => &["nba", "wnba", "nbagame", "kxnbagame", "kxwnbagame"],
        "football" => &["nfl", "ncaaf"],
        "baseball" => &["mlb"],
        "hockey" => &["nhl"],
        "soccer" => &["mls", "epl", "fifa"],
        "knicks" => &["nyk"],
        _ => &[],
    }
}

fn normalize_for_match(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn edit_distance_at_most(a: &str, b: &str, max: usize) -> bool {
    let a_chars = a.chars().collect::<Vec<_>>();
    let b_chars = b.chars().collect::<Vec<_>>();
    let alen = a_chars.len();
    let blen = b_chars.len();
    if alen.abs_diff(blen) > max {
        return false;
    }

    let mut prev = (0..=blen).collect::<Vec<_>>();
    let mut curr = vec![0; blen + 1];

    for i in 1..=alen {
        curr[0] = i;
        let mut row_min = curr[0];
        for j in 1..=blen {
            let cost = usize::from(a_chars[i - 1] != b_chars[j - 1]);
            let del = prev[j] + 1;
            let ins = curr[j - 1] + 1;
            let sub = prev[j - 1] + cost;
            curr[j] = del.min(ins).min(sub);
            row_min = row_min.min(curr[j]);
        }
        if row_min > max {
            return false;
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[blen] <= max
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
                .or_else(|| {
                    data.get("markets")
                        .and_then(Value::as_array)
                        .map(|a| a.len() as i64)
                })
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
