use std::fmt;

use clap::{Args, Subcommand, ValueEnum};
use serde_json::json;

use crate::{
    client::KalshiClient,
    config::ensure_auth,
    output::{extract_array, print_value, render_order_table},
    query::QueryParams,
    AppContext,
};

#[derive(Debug, Clone, ValueEnum)]
enum Side {
    Yes,
    No,
}

#[derive(Debug, Clone, ValueEnum)]
enum Action {
    Buy,
    Sell,
}

#[derive(Debug, Clone, ValueEnum)]
enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, ValueEnum)]
enum Tif {
    Gtc,
    Fok,
    Ioc,
}

#[derive(Debug, Clone, ValueEnum)]
enum OrderStatus {
    Resting,
    Executed,
    Canceled,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Yes => "yes",
            Self::No => "no",
        };
        f.write_str(value)
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        };
        f.write_str(value)
    }
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Limit => "limit",
            Self::Market => "market",
        };
        f.write_str(value)
    }
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Resting => "resting",
            Self::Executed => "executed",
            Self::Canceled => "canceled",
        };
        f.write_str(value)
    }
}

impl fmt::Display for Tif {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Gtc => "good_till_canceled",
            Self::Fok => "fill_or_kill",
            Self::Ioc => "immediate_or_cancel",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Args)]
pub struct OrderCmd {
    #[command(subcommand)]
    command: OrderSubcmd,
}

#[derive(Debug, Clone, Subcommand)]
enum OrderSubcmd {
    Create {
        ticker: String,
        #[arg(long)]
        side: Side,
        #[arg(long)]
        action: Action,
        #[arg(long)]
        count: i64,
        #[arg(long)]
        price: i64,
        #[arg(long = "type", default_value = "limit")]
        order_type: OrderType,
        #[arg(long, default_value = "gtc")]
        tif: Tif,
    },
    Cancel {
        order_id: String,
    },
    CancelAll {
        #[arg(long)]
        ticker: Option<String>,
    },
    Amend {
        order_id: String,
        #[arg(long)]
        price: Option<i64>,
        #[arg(long)]
        count: Option<i64>,
    },
    List {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        status: Option<OrderStatus>,
        #[arg(long, default_value_t = true)]
        compact: bool,
    },
    Get {
        order_id: String,
    },
}

pub async fn run(ctx: &AppContext, cmd: OrderCmd) -> anyhow::Result<()> {
    ensure_auth(&ctx.runtime)?;
    let client = KalshiClient::new(ctx.runtime.clone())?;

    match cmd.command {
        OrderSubcmd::Create {
            ticker,
            side,
            action,
            count,
            price,
            order_type,
            tif,
        } => {
            let mut body = json!({
                "ticker": ticker,
                "side": side.to_string(),
                "action": action.to_string(),
                "count": count,
                "type": order_type.to_string(),
                "time_in_force": tif.to_string(),
            });

            if matches!(side, Side::Yes) {
                body["yes_price"] = json!(price);
            } else {
                body["no_price"] = json!(price);
            }

            let data = client.post_auth("/portfolio/orders", Some(body)).await?;
            print_value(ctx.output_mode, &data)
        }
        OrderSubcmd::Cancel { order_id } => {
            let data = client
                .delete_auth(&format!("/portfolio/orders/{order_id}"), None)
                .await?;
            print_value(ctx.output_mode, &data)
        }
        OrderSubcmd::CancelAll { ticker } => {
            let q = QueryParams::new()
                .insert("status", "resting")
                .optional("ticker", ticker)
                .build_always();

            let existing = client.get_auth("/portfolio/orders", Some(q)).await?;
            let ids: Vec<String> = existing
                .get("orders")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|o| o.get("order_id").and_then(|v| v.as_str()).map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();

            if ids.is_empty() {
                print_value(ctx.output_mode, &json!({"canceled": 0, "message": "No resting orders"}))
            } else {
                let data = client
                    .delete_auth("/portfolio/orders/batched", Some(json!({"ids": ids})))
                    .await?;
                print_value(ctx.output_mode, &data)
            }
        }
        OrderSubcmd::Amend {
            order_id,
            price,
            count,
        } => {
            let mut body = json!({});
            if let Some(p) = price {
                body["yes_price"] = json!(p);
            }
            if let Some(c) = count {
                body["count"] = json!(c);
            }
            let data = client
                .post_auth(&format!("/portfolio/orders/{order_id}/amend"), Some(body))
                .await?;
            print_value(ctx.output_mode, &data)
        }
        OrderSubcmd::List {
            ticker,
            status,
            compact,
        } => {
            let q = QueryParams::new()
                .limit(200)
                .optional("ticker", ticker)
                .optional("status", status.map(|s| s.to_string()))
                .build();

            let data = client.get_auth("/portfolio/orders", q).await?;
            let rows = extract_array(&data, "orders");
            render_order_table(ctx.output_mode, &rows, compact)
        }
        OrderSubcmd::Get { order_id } => {
            let data = client
                .get_auth(&format!("/portfolio/orders/{order_id}"), None)
                .await?;
            print_value(ctx.output_mode, &data)
        }
    }
}
