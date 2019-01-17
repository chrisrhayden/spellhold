extern crate rand;

use std::env;
use std::error::Error;
use std::path::PathBuf;

use spellhold::daemon::main_loop::Daemon;
use spellhold::client::stdin_handle::StdinHandle;
use spellhold::client::tui::TuiApp;

const MAIN_SOCKET: &str = "/tmp/spellholdd_socket";

enum AppArgs {
    Tui,
    None,
    Stdin,
    Daemon,
}

// returns on first found arg
fn cli_args() -> AppArgs {
    for arg in env::args() {
        if arg == "-d" || arg == "--daemon" {
            return AppArgs::Daemon;
        } else if arg == "-s" || arg == "--stdin" {
            return AppArgs::Stdin;
        } else if arg == "-t" || arg == "--tui" {
            return AppArgs::Tui;
        }
    }

    AppArgs::None
}

fn main() {
    match cli_args() {
        AppArgs::Stdin => {
            if let Err(err) = stdin_runner() {
                eprintln!("Cli Intake Error: {}", err);
            }
        }
        AppArgs::Daemon => {
            if let Err(err) = daemon_runner() {
                eprintln!("Daemon Error: {}", err)
            }
        }
        AppArgs::Tui => {
            if let Err(err) = tui_runner() {
                eprintln!("Daemon Error: {}", err)
            } else {
                println!("Good bye")
            }
        }
        AppArgs::None => eprintln!("No or bad cli args given"),
    }
}

fn stdin_runner() -> Result<(), Box<dyn Error>> {
    let stdin_handle = StdinHandle::new(PathBuf::from(MAIN_SOCKET));

    stdin_handle.run(false)
}

fn daemon_runner() -> Result<(), Box<dyn Error>> {
    let mut da = Daemon::default();
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
