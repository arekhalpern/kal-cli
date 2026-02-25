use serde_json::Value;

use super::{
    fmt_cents, fmt_int, get_i64, get_str, left, print_rows, right, standard_table, status_cell,
    truncate, OutputMode,
};

const ORDER_ID_WIDTH: usize = 14;

pub fn render_order_table(mode: OutputMode, rows: &[Value], compact: bool) -> anyhow::Result<()> {
    if mode == OutputMode::Json {
        return print_rows(mode, rows, &[]);
    }

    if compact {
        let mut table = standard_table(&["Order ID", "Ticker", "Side", "Price", "Count", "Status"]);
        for row in rows {
            table.add_row(vec![
                left(truncate(get_str(row, "order_id"), ORDER_ID_WIDTH)),
                left(truncate(get_str(row, "ticker"), 26)),
                left(get_str(row, "side")),
                right(order_price(row)),
                right(order_count(row, "count", "count_fp")),
                status_cell(get_str(row, "status")),
            ]);
        }
        println!("{table}");
        return Ok(());
    }

    let mut table = standard_table(&[
        "Order ID", "Ticker", "Side", "Action", "Price", "Count", "Remain", "Status", "Created",
    ]);
    for row in rows {
        table.add_row(vec![
            left(truncate(get_str(row, "order_id"), ORDER_ID_WIDTH)),
            left(truncate(get_str(row, "ticker"), 30)),
            left(get_str(row, "side")),
            left(get_str(row, "action")),
            right(order_price(row)),
            right(order_count(row, "count", "count_fp")),
            right(order_count(row, "remaining_count", "remaining_count_fp")),
            status_cell(get_str(row, "status")),
            left(truncate(get_str(row, "created_time"), 20)),
        ]);
    }
    println!("{table}");
    Ok(())
}

fn order_price(row: &Value) -> String {
    let side = get_str(row, "side");
    let cents = if side.eq_ignore_ascii_case("no") {
        get_i64(row, "no_price").or_else(|| get_i64(row, "yes_price"))
    } else {
        get_i64(row, "yes_price").or_else(|| get_i64(row, "no_price"))
    };
    fmt_cents(cents)
}

fn order_count(row: &Value, int_key: &str, fp_key: &str) -> String {
    if let Some(v) = get_i64(row, int_key) {
        return fmt_int(Some(v));
    }
    let fp = get_str(row, fp_key);
    if fp != "-" {
        return fp.to_string();
    }
    "-".to_string()
}
