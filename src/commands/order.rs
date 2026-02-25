use std::collections::BTreeMap;

use clap::{Args, Subcommand, ValueEnum};
use serde_json::json;

use crate::{
    client::KalshiClient,
    config::ensure_auth,
    output::{print_value, render_order_table},
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
        #[arg(long, default_value_t = false)]
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
                "side": side_to_str(&side),
                "action": action_to_str(&action),
                "count": count,
                "type": order_type_to_str(&order_type),
                "time_in_force": tif_to_api(&tif),
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
            let mut q = BTreeMap::new();
            q.insert("status".to_string(), "resting".to_string());
            if let Some(t) = ticker {
                q.insert("ticker".to_string(), t);
            }

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
            let mut q = BTreeMap::new();
            q.insert("limit".to_string(), "200".to_string());
            if let Some(t) = ticker {
                q.insert("ticker".to_string(), t);
            }
            if let Some(s) = status {
                q.insert("status".to_string(), status_to_str(&s).to_string());
            }

            let data = client.get_auth("/portfolio/orders", Some(q)).await?;
            let rows = data
                .get("orders")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
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

fn side_to_str(v: &Side) -> &'static str {
    match v {
        Side::Yes => "yes",
        Side::No => "no",
    }
}

fn action_to_str(v: &Action) -> &'static str {
    match v {
        Action::Buy => "buy",
        Action::Sell => "sell",
    }
}

fn order_type_to_str(v: &OrderType) -> &'static str {
    match v {
        OrderType::Limit => "limit",
        OrderType::Market => "market",
    }
}

fn status_to_str(v: &OrderStatus) -> &'static str {
    match v {
        OrderStatus::Resting => "resting",
        OrderStatus::Executed => "executed",
        OrderStatus::Canceled => "canceled",
    }
}

fn tif_to_api(v: &Tif) -> &'static str {
    match v {
        Tif::Gtc => "good_till_canceled",
        Tif::Fok => "fill_or_kill",
        Tif::Ioc => "immediate_or_cancel",
    }
}
