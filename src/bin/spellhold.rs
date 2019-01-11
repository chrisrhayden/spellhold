extern crate rand;

use std::env;
use std::iter;
use std::error::Error;
use std::io::prelude::*;
use std::io::{stdin, BufRead};
use std::os::unix::net::UnixStream;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

use spellhold::daemon::Daemon;

enum AppArgs {
    Tui,
    None,
    Stdin,
    Daemon,
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
            tui_runner();
        }
        AppArgs::None => eprintln!("No or bad cli args given"),
    }
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

fn stdin_runner() -> Result<(), Box<dyn Error>> {
    let log_id = make_id_string()?;

    let mut stream = UnixStream::connect("/tmp/spellholdd_socket")
        .map_err(|err| format!("Error connecting to socket: {}", err))?;

    let conect_id = format!("connect -ID- {}\n", log_id);
    let content_id = format!("{} -ENDID- ", log_id);

    stream.write_all(&conect_id.as_bytes())?;

    for line in stdin().lock().lines() {
        let mut line = line.unwrap();
        // add the \n back in as lines will return without one
        line += "\n";

        line.insert_str(0, &content_id);

        stream.write_all(line.as_bytes())?;
    }

    stream.write_all(b"end")?;

    Ok(())
}

fn make_id_string() -> Result<String, Box<dyn Error>> {
    let mut rng = thread_rng();

    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs()
        .to_string();

    let log_file = iter::repeat_with(|| rng.sample(Alphanumeric))
        .take(10)
        .collect::<String>();

    Ok(format!("{}_{}", since_epoch, log_file))
}

fn daemon_runner() -> Result<(), Box<dyn Error>> {
    let mut da = Daemon::new(false);
    let mut loop_break = true;

    while loop_break {
        loop_break = da.run()?;
    }

    Ok(())
}

fn tui_runner() {
    println!("add the tui");
}
