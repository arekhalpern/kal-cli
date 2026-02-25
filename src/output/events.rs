use serde_json::Value;

use super::{get_str, left, print_rows, standard_table, status_cell, truncate, OutputMode};

pub fn render_events_table(
    mode: OutputMode,
    events: &[Value],
    fallback_status: Option<&str>,
) -> anyhow::Result<()> {
    if mode == OutputMode::Json {
        return print_rows(mode, events, &[]);
    }

    let mut table = standard_table(&["Event", "Category", "Series", "Market Cnt", "Status"]);
    for event in events {
        table.add_row(vec![
            left(truncate(get_str(event, "title"), 52)),
            left(truncate(get_str(event, "category"), 14)),
            left(truncate(get_str(event, "series_ticker"), 18)),
            left(event_market_count(event)),
            status_cell(event_status(event, fallback_status)),
        ]);
    }

    println!("{table}");
    Ok(())
}

pub fn render_events_top_table(
    mode: OutputMode,
    events: &[Value],
    fallback_status: Option<&str>,
) -> anyhow::Result<()> {
    if mode == OutputMode::Json {
        return print_rows(mode, events, &[]);
    }

    let mut table = standard_table(&[
        "Event",
        "Category",
        "Series",
        "Market Cnt",
        "Total Vol",
        "Open Int",
        "Status",
    ]);
    for event in events {
        table.add_row(vec![
            left(truncate(get_str(event, "title"), 44)),
            left(truncate(get_str(event, "category"), 14)),
            left(truncate(get_str(event, "series_ticker"), 16)),
            left(event_market_count(event)),
            super::right(super::fmt_int(super::get_i64(event, "total_volume"))),
            super::right(super::fmt_int(super::get_i64(event, "open_interest"))),
            status_cell(event_status(event, fallback_status)),
        ]);
    }

    println!("{table}");
    Ok(())
}

fn event_status<'a>(event: &'a Value, fallback_status: Option<&'a str>) -> &'a str {
    let status = get_str(event, "status");
    if status == "-" || status.eq_ignore_ascii_case("null") || status.is_empty() {
        return fallback_status.unwrap_or("-");
    }
    status
}

fn event_market_count(event: &Value) -> String {
    if let Some(markets) = event.get("markets").and_then(Value::as_array) {
        return markets.len().to_string();
    }

    for key in ["market_count", "markets_count", "num_markets"] {
        if let Some(v) = event.get(key) {
            if let Some(n) = v.as_i64() {
                return n.to_string();
            }
            if let Some(s) = v.as_str() {
                return s.to_string();
            }
        }
    }

    "-".to_string()
}
