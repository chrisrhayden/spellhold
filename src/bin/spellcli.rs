extern crate rand;

use std::iter;
use std::io::{self, stdin, BufRead};
use std::error::Error;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

fn main() {
    if let Err(err) = run() {
        println!("in main");
        eprintln!("{}", err);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let log_id = make_id_string()?;

    let mut stream = match UnixStream::connect("/tmp/spellholdd_socket") {
        Err(err) => return Err(Box::from(format!("Socket Error: {}", err))),
        Ok(val) => val,
    };


    let conect_id = format!("connect -ID- {}\n", log_id);
    stream.write_all(conect_id.as_bytes())?;
    stream.flush()?;

    let stdin = stdin();
    for line in stdin.lock().lines() {

        let mut line = line.unwrap();

        line += "\n";

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

    let log_file: String = iter::repeat_with(|| rng.sample(Alphanumeric))
        .take(10)
        .collect();

    Ok(format!("{}_{}", since_epoch, log_file))
}
