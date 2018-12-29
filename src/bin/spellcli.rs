extern crate rand;

use std::iter;
use std::io;
use std::io::Write;
use std::io::stdin;
use std::error::Error;
use std::os::unix::net::UnixStream;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let log_id = make_id_string()?;

    let mut stream = UnixStream::connect("/tmp/spellhold_client")?;

    let mut buf_string = String::new();

    loop {
        let line = stdin().read_line(&mut buf_string)?;

        println!("{}", line);

    }

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
