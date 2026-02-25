use serde_json::Value;

use super::{
    OutputMode, fmt_int, get_i64, get_str, left, print_rows, right, standard_table,
    status_cell, truncate,
};

pub fn render_markets_table(
    mode: OutputMode,
    markets: &[Value],
    _compact: bool,
) -> anyhow::Result<()> {
    if mode == OutputMode::Json {
        return print_rows(mode, markets, &[]);
    }

    let mut table = standard_table(&[
        "Question",
        "Contract",
        "Price (Yes %)",
        "Volume",
        "Liquidity",
        "Evt Mkts",
        "Status",
    ]);
    let mut prev_question: Option<String> = None;
    for m in markets {
        let title = get_str(m, "title");
        let question = if title == "-" { get_str(m, "ticker") } else { title };
        let question = clean_question(question);
        let show_question = match &prev_question {
            Some(prev) if prev == question => "",
            _ => question,
        };
        prev_question = Some(question.to_string());

        let yes_price = fmt_percent(
            get_i64(m, "yes_ask")
                .or_else(|| get_i64(m, "yes_bid"))
                .or_else(|| get_i64(m, "last_price")),
        );
        let vol = fmt_int(get_i64(m, "volume"));
        let liq = fmt_int(get_i64(m, "liquidity"));
        let evt_mkts = fmt_int(get_i64(m, "event_market_count"));
        let contract = contract_label(m);

        table.add_row(vec![
            left(truncate(show_question, 40)),
            left(truncate(&contract, 24)),
            right(yes_price),
            right(vol),
            right(liq),
            right(evt_mkts),
            status_cell(get_str(m, "status")),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub fn render_markets_top_table(mode: OutputMode, markets: &[Value]) -> anyhow::Result<()> {
    if mode == OutputMode::Json {
        return print_rows(mode, markets, &[]);
    }

    let mut table = standard_table(&[
        "Question",
        "Contract",
        "Price (Yes %)",
        "Total Vol",
        "Open Int",
        "Evt Mkts",
        "Status",
    ]);
    let mut prev_question: Option<String> = None;
    for m in markets {
        let title = get_str(m, "title");
        let question = if title == "-" { get_str(m, "ticker") } else { title };
        let question = clean_question(question);
        let show_question = match &prev_question {
            Some(prev) if prev == question => "",
            _ => question,
        };
        prev_question = Some(question.to_string());

        let yes_price = fmt_percent(
            get_i64(m, "yes_ask")
                .or_else(|| get_i64(m, "yes_bid"))
                .or_else(|| get_i64(m, "last_price")),
        );
        let total_vol = fmt_int(get_i64(m, "volume"));
        let open_int = fmt_int(get_i64(m, "open_interest"));
        let evt_mkts = fmt_int(get_i64(m, "event_market_count"));
        let contract = contract_label(m);

        table.add_row(vec![
            left(truncate(show_question, 40)),
            left(truncate(&contract, 24)),
            right(yes_price),
            right(total_vol),
            right(open_int),
            right(evt_mkts),
            status_cell(get_str(m, "status")),
        ]);
    }
    println!("{table}");
    Ok(())
}

fn clean_question(input: &str) -> &str {
    let trimmed = input.trim_start();
    if let Some(rest) = trimmed.strip_prefix("yes ") {
        return rest;
    }
    if let Some(rest) = trimmed.strip_prefix("Yes ") {
        return rest;
    }
    if let Some(rest) = trimmed.strip_prefix("no ") {
        return rest;
    }
    if let Some(rest) = trimmed.strip_prefix("No ") {
        return rest;
    }
    trimmed
}

fn fmt_percent(cents: Option<i64>) -> String {
    match cents {
        Some(v) => format!("{v}%"),
        None => "-".to_string(),
    }
}

fn contract_label(market: &Value) -> String {
    for key in ["yes_sub_title", "subtitle", "no_sub_title"] {
        let v = get_str(market, key);
        if v != "-" && !v.is_empty() {
            return clean_question(v).to_string();
        }
    }

    let exp = get_str(market, "expiration_time");
    if exp != "-" && exp.len() >= 10 {
        return format!("Exp {}", &exp[..10]);
    }

    get_str(market, "ticker").to_string()
}
