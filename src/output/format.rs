use comfy_table::{Attribute, Cell, Color};
use serde_json::Value;

use super::right;

pub fn fmt_int(value: Option<i64>) -> String {
    let Some(v) = value else {
        return "-".to_string();
    };
    with_grouping(v)
}

pub fn fmt_cents(value: Option<i64>) -> String {
    let Some(v) = value else {
        return "-".to_string();
    };
    format!("{v}¢")
}

pub fn status_cell(status: &str) -> Cell {
    let normalized = status.to_ascii_lowercase();
    match normalized.as_str() {
        "active" | "open" | "resting" | "executed" => {
            Cell::new(status).fg(Color::Green).add_attribute(Attribute::Bold)
        }
        "closed" | "canceled" => Cell::new(status).fg(Color::Yellow).add_attribute(Attribute::Bold),
        "settled" => Cell::new(status).fg(Color::DarkGrey).add_attribute(Attribute::Bold),
        _ => Cell::new(status).fg(Color::Cyan),
    }
}

pub fn pnl_cell(value: Option<i64>) -> Cell {
    match value {
        Some(v) if v > 0 => right(fmt_cents(Some(v))).fg(Color::Green),
        Some(v) if v < 0 => right(fmt_cents(Some(v))).fg(Color::Red),
        Some(v) => right(fmt_cents(Some(v))),
        None => right("-"),
    }
}

pub fn get_i64(row: &Value, key: &str) -> Option<i64> {
    row.get(key).and_then(|v| match v {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s
            .parse::<i64>()
            .ok()
            .or_else(|| s.parse::<f64>().ok().map(|f| f.round() as i64)),
        _ => None,
    })
}

pub fn get_str<'a>(row: &'a Value, key: &str) -> &'a str {
    row.get(key).and_then(Value::as_str).unwrap_or("-")
}

fn with_grouping(value: i64) -> String {
    let negative = value < 0;
    let digits = value.abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    for (idx, ch) in digits.chars().enumerate() {
        if idx > 0 && (digits.len() - idx) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if negative {
        format!("-{out}")
    } else {
        out
    }
}
