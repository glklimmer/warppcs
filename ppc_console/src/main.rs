use bevy_remote::{BrpRequest, http::DEFAULT_ADDR, http::DEFAULT_PORT};
use clap::{CommandFactory, Parser, Subcommand, ValueHint, arg, command};
use clap_complete::{Shell, generate};
use console_protocol::*;
use dialoguer::Select;
use serde_json::{Value, to_value};
use std::{fs::File, path::PathBuf};

#[derive(Parser)]
#[command(name = "ppc", about = "The warppc first tooling", version = "0.1.0")]
pub struct PPC {
    #[arg(long, default_value = "DEFAULT_ADDR.to_string()")]
    host: String,

    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,

    #[command(subcommand)]
    pub command: PPCSubCommands,
}

#[derive(Subcommand)]
pub enum PPCSubCommands {
    RandomItem {},
    SpawnUnit {
        #[arg(short, long, value_hint = ValueHint::CommandWithArguments)]
        unit: Option<String>,

        #[arg(short, long, value_hint = ValueHint::CommandWithArguments, default_value_t = 0)]
        player: u8,
    },
}

fn main() {
    let cli = PPC::parse();

    let host_part = format!("{}:{}", DEFAULT_ADDR.to_string(), DEFAULT_PORT);
    let url = format!("http://{}/", host_part);
    let home_dir = dirs::home_dir().expect("Could not find home directory");

    let mut zsh_completion_dir = PathBuf::from(home_dir);
    zsh_completion_dir.push(".zsh");
    zsh_completion_dir.push("completion");
    zsh_completion_dir.push("ppc");
    let mut output_file =
        File::create(&zsh_completion_dir).expect(&format!("Could not create file: "));

    generate(Shell::Zsh, &mut PPC::command(), "ppc", &mut output_file);

    match cli.command {
        PPCSubCommands::RandomItem {} => {
            let req = BrpRequest {
                jsonrpc: String::from("2.0"),
                method: BRP_TRIGGER_RANDOM_ITEM.into(),
                id: None,
                params: None,
            };
            let res = ureq::post(&url)
                .send_json(req)
                .unwrap()
                .body_mut()
                .read_json::<Value>()
                .unwrap();

            println!("{:#}", res);
        }
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

            let req = BrpRequest {
                jsonrpc: String::from("2.0"),
                method: BRP_TRIGGER_SPAWN_UNIT.into(),
                id: None,
                params: Some(
                    to_value(BrpSpawnUnit {
                        unit: unit_spawn,
                        player,
                    })
                    .expect("Unable to convert query parameters to a valid JSON value"),
                ),
            };
            let res = ureq::post(&url)
                .send_json(req)
                .unwrap()
                .body_mut()
                .read_json::<Value>()
                .unwrap();

            println!("{:#}", res);
        }
    };
}
