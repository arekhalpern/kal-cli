use serde_json::Value;

use super::{
    fmt_int, get_i64, get_str, left, pnl_cell, print_rows, print_value, right, standard_table,
    status_cell, truncate, OutputMode,
};

pub fn render_balance_table(mode: OutputMode, data: &Value) -> anyhow::Result<()> {
    if mode == OutputMode::Json {
        return print_value(mode, data);
    }

    let Some(obj) = data.as_object() else {
        return print_value(mode, data);
    };

    let mut table = standard_table(&["Field", "Value"]);
    for (key, value) in obj {
        if !should_show_balance_field(key) {
            continue;
        }
        table.add_row(vec![left(key), right(balance_cell_value(key, value))]);
    }
    println!("{table}");
    Ok(())
}

pub fn render_positions_table(
    mode: OutputMode,
    rows: &[Value],
    compact: bool,
) -> anyhow::Result<()> {
    if mode == OutputMode::Json {
        return print_rows(mode, rows, &[]);
    }

    if compact {
        let mut table = standard_table(&["Ticker", "Position", "PnL", "Status"]);
        for row in rows {
            let status = row
                .get("market_result")
                .and_then(Value::as_str)
                .map(|_| "settled")
                .unwrap_or("open");
            table.add_row(vec![
                left(truncate(get_str(row, "ticker"), 30)),
                right(fmt_int(get_i64(row, "position"))),
                pnl_cell(get_i64(row, "realized_pnl")),
                status_cell(status),
            ]);
        }
        println!("{table}");
        return Ok(());
    }

    let mut table = standard_table(&["Ticker", "Market", "Pos", "PnL", "Fees", "Event"]);
    for row in rows {
        table.add_row(vec![
            left(truncate(get_str(row, "ticker"), 28)),
            left(truncate(get_str(row, "market_title"), 44)),
            right(fmt_int(get_i64(row, "position"))),
            pnl_cell(get_i64(row, "realized_pnl")),
            right(fmt_int(get_i64(row, "fees_paid"))),
            left(truncate(get_str(row, "event_title"), 36)),
        ]);
    }
    println!("{table}");
    Ok(())
}

fn balance_cell_value(key: &str, value: &Value) -> String {
    if matches!(key, "balance" | "portfolio_value") {
        return fmt_dollars(value);
    }
    scalar_to_string(value)
}

fn should_show_balance_field(key: &str) -> bool {
    !matches!(key, "updated_ts" | "update_ts")
}

fn fmt_dollars(value: &Value) -> String {
    let Some(cents) = value_to_cents(value) else {
        return scalar_to_string(value);
    };
    let sign = if cents < 0 { "-" } else { "" };
    let abs = (cents as i128).abs();
    format!("{sign}${}.{:02}", abs / 100, abs % 100)
}

fn value_to_cents(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n
            .as_i64()
            .or_else(|| n.as_f64().map(|f| f.round() as i64)),
        Value::String(s) => s
            .parse::<i64>()
            .ok()
            .or_else(|| s.parse::<f64>().ok().map(|f| f.round() as i64)),
        _ => None,
    }
}

fn scalar_to_string(value: &Value) -> String {
    match value {
        Value::Null => "-".to_string(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "<unprintable>".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{balance_cell_value, should_show_balance_field};

    #[test]
    fn formats_balance_in_dollars() {
        assert_eq!(balance_cell_value("balance", &json!(1110)), "$11.10");
        assert_eq!(balance_cell_value("portfolio_value", &json!("15007")), "$150.07");
    }

    #[test]
    fn leaves_non_balance_fields_unchanged() {
        assert_eq!(balance_cell_value("updated_ts", &json!(1771987869)), "1771987869");
    }

    #[test]
    fn hides_balance_timestamp_fields() {
        assert!(!should_show_balance_field("updated_ts"));
        assert!(!should_show_balance_field("update_ts"));
        assert!(should_show_balance_field("balance"));
    }
}
