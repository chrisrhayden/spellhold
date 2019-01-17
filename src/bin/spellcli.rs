extern crate clap;
extern crate rand;

use std::error::Error;
use std::path::PathBuf;

use clap::{Arg, App, SubCommand};

use spellhold::daemon::main_loop::Daemon;
use spellhold::client::stdin_handle::StdinHandle;
use spellhold::client::tui::TuiApp;

const MAIN_SOCKET: &str = "/tmp/spellholdd_socket";

enum AppAction {
    None,
    Tui,
    Daemon,
    Stdin,
}

struct AppArgs {
    quite: bool,
    action: AppAction,
    optional_value: Option<String>,
}

impl AppArgs {
    fn new() -> AppArgs {
        let matches = App::new("spellcli")
            .arg(
                Arg::with_name("quite")
                    .short("q")
                    .long("quite")
                    .value_name("BOOL")
                    .takes_value(false)
                    .help("whether should run quite"),
            )
            .subcommand(
                SubCommand::with_name("daemon")
                    .help("run the spellhold daemon")
                    .visible_alias("d")
                    .arg(
                        Arg::with_name("daemon socket")
                            .requires("daemon")
                            .short("p")
                            .long("daemon-path")
                            .value_name("DAEMON_PATH")
                            .takes_value(true)
                            .help("the daemon path"),
                    ),
            )
            .subcommand(
                SubCommand::with_name("stdin")
                    .help("take stdin and sent it to the daemon")
                    .visible_alias("s")
                    .arg(
                        Arg::with_name("stdin name")
                            .requires("stdin")
                            .short("n")
                            .long("std-name")
                            .value_name("STDIN_NAME")
                            .takes_value(true)
                            .help("the stdin name"),
                    ),
            )
            .subcommand(
                SubCommand::with_name("tui")
                    .help("run the tui")
                    .visible_alias("t"),
            )
            .get_matches();

        let quite = match matches
            .value_of("quite")
            .unwrap_or("true")
            .to_lowercase()
            .as_ref()
        {
            "true" => true,
            "false" => false,
            _ => true,
        };

        let (action, optional_value) = if matches.is_present("daemon") {
            let opt =
                Some(matches.value_of("daemon").unwrap_or("").to_string());

            (AppAction::Daemon, opt)
        } else if matches.is_present("stdin") {
            let opt = Some(matches.value_of("stdin").unwrap_or("").to_string());

            (AppAction::Stdin, opt)
        } else if matches.is_present("tui") {
            (AppAction::Tui, None)
        } else {
            (AppAction::None, None)
        };

        AppArgs {
            quite,
            action,
            optional_value,
        }
    }
}

fn main() {
    let app = AppArgs::new();

    match app.action {
        AppAction::Stdin => {
            if let Err(err) = stdin_runner(app.quite, app.optional_value) {
                eprintln!("Cli Intake Error: {}", err);
            }
        }
        AppAction::Daemon => {
            if let Err(err) = daemon_runner(app.quite, app.optional_value) {
                eprintln!("Daemon Error: {}", err)
            }
        }
        AppAction::Tui => {
            if let Err(err) = tui_runner() {
                eprintln!("Daemon Error: {}", err)
            } else {
                println!("Good bye")
            }
        }
        AppAction::None => eprintln!("No or bad cli args given"),
    }
}

fn stdin_runner(
    quite: bool,
    name: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let stdin_handle = StdinHandle::new(PathBuf::from(MAIN_SOCKET), quite);

    stdin_handle.run(name)
}

fn daemon_runner(
    quit: bool,
    socket: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let mut da = Daemon::new(quit, socket);

    let mut loop_break = true;

    while loop_break {
        loop_break = da.run()?;
    }

    Ok(())
}

fn tui_runner() -> Result<(), Box<dyn Error>> {
    let mut tui = TuiApp::new(PathBuf::from(MAIN_SOCKET));

    tui.run()
}
