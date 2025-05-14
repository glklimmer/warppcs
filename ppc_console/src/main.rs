use bevy_remote::{BrpRequest, http::DEFAULT_ADDR, http::DEFAULT_PORT};
use clap::{Parser, Subcommand, ValueHint, arg, command};
use console_protocol::*;
use dialoguer::Select;
use serde_json::{Value, to_value};

#[derive(Parser)]
#[command(name = "ppc", about = "Cheat console for warppcs.", version = "0.1.0")]
pub struct PPC {
    #[arg(long, default_value_t = DEFAULT_ADDR.to_string())]
    host: String,

    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,

    #[command(subcommand)]
    pub command: PPCSubCommands,
}

#[derive(Subcommand)]
pub enum PPCSubCommands {
    RandomItems {
        #[arg(short, long, value_hint = ValueHint::CommandWithArguments, default_value_t = 0)]
        player: u8,
    },
    SpawnUnit {
        #[arg(short, long, value_hint = ValueHint::CommandWithArguments)]
        unit: Option<String>,

        #[arg(short, long, value_hint = ValueHint::CommandWithArguments, default_value_t = 0)]
        player: u8,
    },
}

fn main() {
    let cli = PPC::parse();

    let host_part = format!("{}:{}", DEFAULT_ADDR, DEFAULT_PORT);
    let url = format!("http://{}/", host_part);

    let request = match cli.command {
        PPCSubCommands::RandomItems { player } => BrpRequest {
            jsonrpc: String::from("2.0"),
            method: BRP_SPAWN_RANDOM_ITEM.into(),
            id: None,
            params: Some(
                to_value(BrpSpawnItems { player })
                    .expect("Unable to convert query parameters to a valid JSON value"),
            ),
        },
        PPCSubCommands::SpawnUnit { unit, player } => {
            let unit_spawn = match unit {
                Some(unit) => unit,
                None => {
                    let items = vec!["archer", "pikemen", "shield"];
                    let selection = Select::new()
                        .with_prompt("Which unit?")
                        .items(&items)
                        .interact()
                        .unwrap();

                    items[selection].to_string()
                }
            };

            BrpRequest {
                jsonrpc: String::from("2.0"),
                method: BRP_SPAWN_UNIT.into(),
                id: None,
                params: Some(
                    to_value(BrpSpawnUnit {
                        unit: unit_spawn,
                        player,
                    })
                    .expect("Unable to convert query parameters to a valid JSON value"),
                ),
            }
        }
    };

    let response = ureq::post(&url)
        .send_json(request)
        .unwrap()
        .body_mut()
        .read_json::<Value>()
        .unwrap();

    println!("{:#}", response);
}
