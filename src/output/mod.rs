mod events;
mod format;
mod markets;
mod orders;
mod portfolio;
mod table;

pub use events::{render_events_table, render_events_top_table};
pub use format::{extract_array, fmt_cents, fmt_int, get_i64, get_str, pnl_cell, status_cell};
pub use markets::{render_markets_table, render_markets_top_table};
pub use orders::render_order_table;
pub use portfolio::render_positions_table;
pub use table::{left, right, standard_table, truncate};

use comfy_table::{presets::UTF8_FULL, Cell, Table};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Table,
    Json,
}

pub fn print_value(mode: OutputMode, value: &Value) -> anyhow::Result<()> {
    match mode {
        OutputMode::Json => {
            println!("{}", serde_json::to_string_pretty(value)?);
        }
        OutputMode::Table => {
            if let Some(obj) = value.as_object() {
                let mut table = Table::new();
                table.load_preset(UTF8_FULL);
                table.set_header(vec!["Field", "Value"]);
                for (k, v) in obj {
                    table.add_row(vec![Cell::new(k), Cell::new(short_json(v))]);
                }
                println!("{table}");
            } else {
                println!("{}", serde_json::to_string_pretty(value)?);
            }
        }
    }
    Ok(())
}

pub fn print_rows(mode: OutputMode, rows: &[Value], columns: &[&str]) -> anyhow::Result<()> {
    match mode {
        OutputMode::Json => {
            println!("{}", serde_json::to_string_pretty(rows)?);
        }
        OutputMode::Table => {
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(columns.iter().map(|c| Cell::new(*c)).collect::<Vec<_>>());

            for row in rows {
                table.add_row(
                    columns
                        .iter()
                        .map(|c| Cell::new(read_column(row, c)))
                        .collect::<Vec<_>>(),
                );
            }

            println!("{table}");
        }
    }
    Ok(())
}

pub fn print_ndjson(value: &Value) {
    println!(
        "{}",
        serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
    );
}

fn read_column(value: &Value, key: &str) -> String {
    match value.get(key) {
        Some(v) => short_json(v),
        None => "-".to_string(),
    }
}

fn short_json(value: &Value) -> String {
    match value {
        Value::Null => "-".to_string(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "<unprintable>".to_string()),
    }
}
