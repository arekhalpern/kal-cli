use clap::{Args, Subcommand};
use dialoguer::{Confirm, Input, Select};
use serde_json::json;

use crate::{
    config::{config_path, delete_config, load_stored_config, save_config, Environment, StoredConfig},
    output::{print_value, OutputMode},
};

#[derive(Debug, Clone, Args)]
pub struct ConfigCmd {
    #[command(subcommand)]
    command: ConfigSubcmd,
}

#[derive(Debug, Clone, Subcommand)]
enum ConfigSubcmd {
    Setup,
    Show,
    Path,
    Reset,
}

pub async fn run(cmd: ConfigCmd, mode: OutputMode) -> anyhow::Result<()> {
    match cmd.command {
        ConfigSubcmd::Setup => setup(mode),
        ConfigSubcmd::Show => show(mode),
        ConfigSubcmd::Path => path(mode),
        ConfigSubcmd::Reset => reset(mode),
    }
}

fn setup(mode: OutputMode) -> anyhow::Result<()> {
    let env_choices = ["prod", "demo"];
    let env_idx = Select::new()
        .with_prompt("Environment")
        .items(&env_choices)
        .default(0)
        .interact()?;

    let api_key: String = Input::new().with_prompt("API key").interact_text()?;
    let api_secret_path: String = Input::new()
        .with_prompt("API secret path (PEM)")
        .interact_text()?;

    let cfg = StoredConfig {
        api_key: Some(api_key),
        api_secret_path: Some(api_secret_path),
        environment: Some(if env_idx == 0 {
            Environment::Prod
        } else {
            Environment::Demo
        }),
    };

    save_config(&cfg)?;
    print_value(mode, &json!({"ok": true, "path": config_path()?.display().to_string()}))
}

fn show(mode: OutputMode) -> anyhow::Result<()> {
    let cfg = load_stored_config()?;
    let masked = json!({
        "apiKey": cfg.api_key.as_ref().map(mask_secret),
        "apiSecretPath": cfg.api_secret_path,
        "environment": cfg.environment.map(|e| match e { Environment::Prod => "prod", Environment::Demo => "demo" }),
    });
    print_value(mode, &masked)
}

fn path(mode: OutputMode) -> anyhow::Result<()> {
    print_value(mode, &json!({"path": config_path()?.display().to_string()}))
}

fn reset(mode: OutputMode) -> anyhow::Result<()> {
    if Confirm::new()
        .with_prompt("Delete config file?")
        .default(false)
        .interact()?
    {
        delete_config()?;
        print_value(mode, &json!({"deleted": true}))
    } else {
        print_value(mode, &json!({"deleted": false}))
    }
}

fn mask_secret(value: &String) -> String {
    if value.len() <= 6 {
        return "***".to_string();
    }
    format!("{}***{}", &value[..3], &value[value.len() - 3..])
}
