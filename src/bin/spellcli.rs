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
    optional_values: Vec<Option<String>>,
}

impl AppArgs {
    fn new() -> AppArgs {
        let matches = App::new("spellcli")
            .about("pip or retrieve stdin from a daemon")
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
                    .visible_alias("d")
                    .about("run the spellhold daemon")
                    .arg(
                        Arg::with_name("daemon socket")
                            .short("s")
                            .long("daemon-socket")
                            .value_name("DAEMON_SOCKET")
                            .takes_value(true)
                            .help("define the daemon socket path"),
                    ),
            )
            .subcommand(
                SubCommand::with_name("stdin")
                    .visible_alias("s")
                    .about("take stdin and send it to the daemon")
                    .arg(
                        Arg::with_name("stdin name")
                            .short("n")
                            .long("std-name")
                            .value_name("STDIN_NAME")
                            .takes_value(true)
                            .help("the stdin name for the tui and log file"),
                    )
                    .arg(
                        Arg::with_name("stdin socket")
                            .short("s")
                            .long("std-socket")
                            .value_name("STDIN_SOCKET")
                            .takes_value(true)
                            .help("the stdin socket if changed from default"),
                    ),
            )
            .subcommand(
                SubCommand::with_name("tui")
                    .visible_alias("t")
                    .about("run the tui")
                    .arg(
                        Arg::with_name("tui socket")
                            .short("T")
                            .long("tui-socket")
                            .value_name("TUI_SOCKET")
                            .takes_value(true)
                            .help("the tui socket to different"),
                    ),
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

        let (action, optional_values) = if matches.is_present("daemon") {
            let opt = matches.value_of("stdin-name").map(String::from);

            (AppAction::Daemon, vec![opt])
        } else if matches.is_present("stdin") {
            let socket = matches.value_of("stdin-socket").map(String::from);
            let name = matches.value_of("stdin-name").map(String::from);

            (AppAction::Stdin, vec![socket, name])
        } else if matches.is_present("tui") {
            (AppAction::Tui, vec![None])
        } else {
            (AppAction::None, vec![None])
        };

        AppArgs {
            quite,
            action,
            optional_values,
        }
    }
}

fn main() {
    let mut app = AppArgs::new();

    match app.action {
        AppAction::Stdin => {
            if let Err(err) = stdin_runner(&app) {
                eprintln!("Cli Intake Error: {}", err);
            }
        }
        AppAction::Daemon => {
            if let Err(err) = daemon_runner(&app) {
                eprintln!("Daemon Error: {}", err)
            }
        }
        AppAction::Tui => {
            if let Err(err) = tui_runner(&mut app) {
                eprintln!("Daemon Error: {}", err)
            } else {
                println!("Good bye")
            }
        }
        AppAction::None => eprintln!("No or bad cli args given"),
    }
}

fn stdin_runner(app: &AppArgs) -> Result<(), Box<dyn Error>> {
    let quite = app.quite;
    let name = app.optional_values[1].to_owned();

    let socket = app.optional_values[0]
        .to_owned()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(MAIN_SOCKET));

    let stdin_handle = StdinHandle::new(socket, quite);

    stdin_handle.run(name)
}

fn daemon_runner(app: &AppArgs) -> Result<(), Box<dyn Error>> {
    let (socket, quite) = (app.optional_values[0].to_owned(), app.quite);

    let mut da = Daemon::new(socket, quite);

    let mut loop_break = true;

    while loop_break {
        if !quite {
            println!("running daemon");
        }

        loop_break = da.run()?;
    }

    Ok(())
}

fn tui_runner(app: &mut AppArgs) -> Result<(), Box<dyn Error>> {
    let socket = if app.optional_values[0].is_some() {
        PathBuf::from(app.optional_values[0].clone().unwrap())
    } else {
        PathBuf::from(MAIN_SOCKET)
    };

    let mut tui = TuiApp::new(socket);

    tui.run()
}
