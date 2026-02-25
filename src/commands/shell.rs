use clap::Parser;
use rustyline::DefaultEditor;

use crate::{dispatch, Cli, Commands};
use crate::output::OutputMode;

pub async fn run(output_mode: OutputMode) -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;

    println!("Kalshi interactive shell. Type 'help' for commands, 'exit' to quit.");

    loop {
        let line = rl.readline("kal> ");
        match line {
            Ok(raw) => {
                let input = raw.trim();
                if input.is_empty() {
                    continue;
                }
                if matches!(input, "exit" | "quit") {
                    break;
                }
                if input == "help" {
                    print_shell_help();
                    continue;
                }

                let _ = rl.add_history_entry(input);
                let tokens = match shell_words::split(input) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("parse error: {e}");
                        continue;
                    }
                };

                let mut argv = vec!["kal".to_string(), "-o".to_string(), match output_mode {
                    OutputMode::Table => "table".to_string(),
                    OutputMode::Json => "json".to_string(),
                }];
                argv.extend(tokens);

                match Cli::try_parse_from(argv) {
                    Ok(cli) => {
                        if matches!(&cli.command, Commands::Shell) {
                            eprintln!("already in shell mode");
                            continue;
                        }
                        // Box::pin to avoid infinite-size future from recursion
                        if let Err(err) = Box::pin(dispatch(cli)).await {
                            eprintln!("error: {err}");
                        }
                    }
                    Err(e) => {
                        eprintln!("{e}");
                    }
                }
            }
            Err(_) => break,
        }
    }

    Ok(())
}

fn print_shell_help() {
    println!("Commands:");
    println!("  markets list|get|search|orderbook");
    println!("  events list|get|top");
    println!("  order create|cancel|amend|list|get|cancel-all");
    println!("  portfolio balance|positions|fills|settlements");
    println!("  trades list");
    println!("  exchange status|schedule|announcements");
    println!("  watch ticker|orderbook|trades");
    println!("  config setup|show|path|reset");
    println!("  help    Show this help");
    println!("  exit    Quit the shell");
}
