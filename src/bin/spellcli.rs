extern crate rand;

use std::iter;
use std::io::stdin;
use std::error::Error;
use std::io::prelude::*;
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

    let mut stream = UnixStream::connect("/tmp/spellholdd_socket")?;

    let mut buf_string = String::new();

    let conect_id = format!("connect -ID- {}\n", log_id);

    stream.write_all(conect_id.as_bytes())?;
    stream.flush()?;

    loop {
        if let Err(err) = stdin().read_line(&mut buf_string) {
            eprintln!("Error {}", err);
            break;
        };

        println!("{}", buf_string);
        stream.write_all(buf_string.as_bytes())?;
        stream.flush()?;
        buf_string.clear();
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
