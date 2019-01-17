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
                            .short("n")
                            .long("std-name")
                            .value_name("STDIN_NAME")
                            .takes_value(true)
                            .help("the stdin name"),
                    )
                    .arg(
                        Arg::with_name("stdin socket")
                            .short("s")
                            .long("std-socket")
                            .value_name("SOCKET_PATH")
                            .takes_value(true)
                            .help("the stdin socket if changed from default"),
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
    let app = AppArgs::new();

    match app.action {
        AppAction::Stdin => {
            let (socket, name) = (
                app.optional_values[0].to_owned(),
                app.optional_values[1].to_owned(),
            );
            if let Err(err) = stdin_runner(socket, app.quite, name) {
                eprintln!("Cli Intake Error: {}", err);
            }
        }
        AppAction::Daemon => {
            if let Err(err) =
                daemon_runner(app.optional_values[0].to_owned(), app.quite)
            {
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
    socket: Option<String>,
    quite: bool,
    name: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let socket: PathBuf = if socket.is_some() {
        PathBuf::from(socket.unwrap())
    } else {
        PathBuf::from(MAIN_SOCKET)
    };

    let stdin_handle = StdinHandle::new(socket, quite);

    stdin_handle.run(name)
}

fn daemon_runner(
    socket: Option<String>,
    quite: bool,
) -> Result<(), Box<dyn Error>> {
    let mut da = Daemon::new(socket, quite);

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
