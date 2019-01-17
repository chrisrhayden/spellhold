extern crate clap;
extern crate rand;

use std::error::Error;
use std::path::PathBuf;

use clap::{Arg, App};

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
    optional_value: String,
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
            .arg(
                Arg::with_name("daemon")
                    .short("d")
                    .long("daemon")
                    .help("optional socket path"),
            )
            .arg(
                Arg::with_name("stdin")
                    .short("s")
                    .long("stdin")
                    .help("take stdin"),
            )
            .arg(
                Arg::with_name("tui")
                    .short("t")
                    .long("tui")
                    .help("run the tui"),
            )
            .arg(
                Arg::with_name("stdin name")
                    .requires("stdin")
                    .short("n")
                    .long("std-name")
                    .value_name("STDIN_NAME")
                    .takes_value(true)
                    .help("the stdin name"),
            )
            .arg(
                Arg::with_name("daemon socket")
                    .requires("daemon")
                    .short("p")
                    .long("daemon-path")
                    .value_name("DAEMON_PATH")
                    .takes_value(true)
                    .help("the daemon path"),
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
            (
                AppAction::Daemon,
                matches.value_of("daemon socket").unwrap_or("").to_string(),
            )
        } else if matches.is_present("stdin") {
            (
                AppAction::Stdin,
                matches.value_of("stdin name").unwrap_or("").to_string(),
            )
        } else if matches.is_present("tui") {
            (AppAction::Tui, String::new())
        } else {
            (AppAction::None, String::new())
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
            if let Err(err) = stdin_runner(app.quite, &app.optional_value) {
                eprintln!("Cli Intake Error: {}", err);
            }
        }
        AppAction::Daemon => {
            if let Err(err) = daemon_runner(app.quite) {
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

fn stdin_runner(quite: bool, optional: &str) -> Result<(), Box<dyn Error>> {
    let stdin_handle = StdinHandle::new(PathBuf::from(MAIN_SOCKET));

    println!("cmd name {}", optional);
    stdin_handle.run(quite)
}

fn daemon_runner(quit: bool) -> Result<(), Box<dyn Error>> {
    let mut da = Daemon::new(quit, None);

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
