mod commands;
mod utils;

use clap::{App, Arg, SubCommand};
use std::process;

fn main() {
    let matches = App::new("ncy")
        .version("0.1.0")
        .author("Your Name <your.email@example.com>")
        .about("A CLI PKM (Personal Knowledge Management) tool")
        .subcommand(SubCommand::with_name("init").about("Initialize and configure ncy"))
        .subcommand(
            SubCommand::with_name("set")
                .about("Set the default vault")
                .arg(
                    Arg::with_name("vault")
                        .help("Name of the vault to set as default")
                        .required(true),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("init", Some(_)) => {
            if let Err(e) = commands::init::execute() {
                eprintln!("Application error: {}", e);
                process::exit(1);
            }
        }
        ("set", Some(set_matches)) => {
            let vault_name = set_matches.value_of("vault").unwrap();
            if let Err(e) = commands::set::execute(vault_name) {
                eprintln!("Application error: {}", e);
                process::exit(1);
            }
        }
        // Default action when no subcommand is specified
        _ => {
            if let Err(e) = commands::open::execute() {
                eprintln!("Application error: {}", e);
                process::exit(1);
            }
        }
    }
}
