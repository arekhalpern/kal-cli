use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Args, Subcommand};

use crate::{
    client::KalshiClient,
    config::ensure_auth,
    output::{extract_array, print_rows, print_value, render_positions_table},
    query::QueryParams,
    AppContext,
};

#[derive(Debug, Clone, Args)]
pub struct PortfolioCmd {
    #[command(subcommand)]
    command: PortfolioSubcmd,
}

#[derive(Debug, Clone, Subcommand)]
enum PortfolioSubcmd {
    Balance,
    Positions {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long = "event")]
        event_ticker: Option<String>,
        #[arg(long, default_value_t = false)]
        settled: bool,
        #[arg(long, default_value_t = false)]
        unsettled: bool,
        #[arg(long, default_value_t = true)]
        compact: bool,
    },
    Fills {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long, default_value_t = 7)]
        days: u64,
    },
    Settlements {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long, default_value_t = 30)]
        days: u64,
    },
}

pub async fn run(ctx: &AppContext, cmd: PortfolioCmd) -> anyhow::Result<()> {
    ensure_auth(&ctx.runtime)?;
    let client = KalshiClient::new(ctx.runtime.clone())?;

    match cmd.command {
        PortfolioSubcmd::Balance => {
            let data = client.get_auth("/portfolio/balance", None).await?;
            print_value(ctx.output_mode, &data)
        }
        PortfolioSubcmd::Positions {
            ticker,
            event_ticker,
            settled,
            unsettled,
            compact,
        } => {
            let settlement_status = if settled {
                Some("settled")
            } else if unsettled {
                Some("unsettled")
            } else {
                None
            };
            let q = QueryParams::new()
                .limit(200)
                .optional("ticker", ticker)
                .optional("event_ticker", event_ticker)
                .optional("settlement_status", settlement_status)
                .build();

            let data = client.get_auth("/portfolio/positions", q).await?;
            let rows = extract_array(&data, "market_positions");
            render_positions_table(ctx.output_mode, &rows, compact)
        }
        PortfolioSubcmd::Fills { ticker, days } => {
            let q = QueryParams::new()
                .limit(200)
                .insert("min_ts", ts_days_ago(days))
                .optional("ticker", ticker)
                .build();

            let data = client.get_auth("/portfolio/fills", q).await?;
            let rows = extract_array(&data, "fills");
            print_rows(
                ctx.output_mode,
                &rows,
                &[
                    "trade_id",
                    "ticker",
                    "side",
                    "action",
                    "count",
                    "yes_price",
                    "created_time",
                ],
            )
        }
        PortfolioSubcmd::Settlements { ticker, days } => {
            let q = QueryParams::new()
                .limit(200)
                .insert("min_ts", ts_days_ago(days))
                .optional("ticker", ticker)
                .build();

            let data = client.get_auth("/portfolio/settlements", q).await?;
            let rows = extract_array(&data, "settlements");
            print_rows(
                ctx.output_mode,
                &rows,
                &[
                    "ticker",
                    "market_result",
                    "realized_pnl",
                    "yes_count",
                    "no_count",
                    "settlement_ts",
                ],
            )
        }
    }
}

fn ts_days_ago(days: u64) -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    now - (days as i64 * 86_400)
}
