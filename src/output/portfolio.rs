use serde_json::Value;

use super::{
    OutputMode, fmt_int, get_i64, get_str, left, pnl_cell, print_rows, right, standard_table,
    status_cell, truncate,
};

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
