mod commands;
mod utils;

use clap::{App, Arg, SubCommand};
use std::process;

fn main() {
    let matches = App::new("ncy")
        .version("0.1.0")
        .author("Your Name <your.email@example.com>")
        .about("A CLI PKM (Personal Knowledge Management) tool")
        .arg(
            Arg::with_name("external")
                .short("e")
                .long("external")
                .help("Use fzf for picking notes instead of nucleo_picker (useful for integration with text editors)")
                .takes_value(false),
        )
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
        .subcommand(
            SubCommand::with_name("new")
                .visible_alias("n")
                .about("Create a new note")
                .arg(
                    Arg::with_name("external")
                        .short("e")
                        .long("external")
                        .help("Create file and print absolute path to stdout (useful for integration with text editors)")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("args")
                        .help("Note title, project path, and vault in format: 'title @ project/path +vault'")
                        .required(true)
                        .multiple(true), // Allow multiple arguments to be combined into one string
                ),
        )
        .subcommand(
            SubCommand::with_name("jrnl")
                .visible_alias("j")
                .about("Open or add to today's journal entry")
                .arg(
                    Arg::with_name("external")
                        .short("e")
                        .long("external")
                        .help("Print the absolute path of today's journal file instead of opening it")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("text")
                        .help("Text to add to the journal entry (if not provided, opens today's entry)")
                        .required(false)
                        .multiple(true), // Allow multiple arguments to be combined into one string
                ),
        )
        .subcommand(
            SubCommand::with_name("dir")
                .visible_alias("d")
                .about("Select a note and open its directory in the file explorer")
                .arg(
                    Arg::with_name("external")
                        .short("e")
                        .long("external")
                        .help("Print directory path to stdout rather than opening in file explorer")
                        .takes_value(false),
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
        ("new", Some(new_matches)) | ("n", Some(new_matches)) => {
            // Collect all arguments into a single string
            let args: Vec<&str> = new_matches.values_of("args").unwrap().collect();
            let combined_args = args.join(" ");

            // Check if external flag is set
            let use_external = new_matches.is_present("external");

            if let Err(e) = commands::new::execute_with_options(&combined_args, use_external) {
                eprintln!("Application error: {}", e);
                process::exit(1);
            }
        }
        ("jrnl", Some(jrnl_matches)) | ("j", Some(jrnl_matches)) => {
            // Get the external flag
            let external = jrnl_matches.is_present("external");

            // Check if any text was provided
            let combined_args = if let Some(values) = jrnl_matches.values_of("text") {
                let args: Vec<&str> = values.collect();
                args.join(" ")
            } else {
                String::new() // Empty string if no text provided
            };

            if let Err(e) = commands::jrnl::execute(&combined_args, external) {
                eprintln!("Application error: {}", e);
                process::exit(1);
            }
        }
        ("dir", Some(dir_matches)) | ("d", Some(dir_matches)) => {
            // Get the external flag
            let use_external = dir_matches.is_present("external");

            if let Err(e) = commands::dir::execute_with_options(use_external) {
                eprintln!("Application error: {}", e);
                process::exit(1);
            }
        }
        // Default action when no subcommand is specified
        _ => {
            let use_external = matches.is_present("external");
            if let Err(e) = commands::open::execute_with_options(use_external) {
                eprintln!("Application error: {}", e);
                process::exit(1);
            }
        }
    }
}
