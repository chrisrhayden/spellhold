use std::iter;
use std::error::Error;
use std::path::PathBuf;
use std::os::unix::net::UnixStream;
use std::io::{stdin, BufRead, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

fn make_id_string(name: Option<String>) -> Result<String, Box<dyn Error>> {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs()
        .to_string();

    let log_file = if name.is_some() {
        name.unwrap()
    } else {
        let mut rng = thread_rng();
        iter::repeat_with(|| rng.sample(Alphanumeric))
            .take(10)
            .collect::<String>()
    };

    Ok(format!("{}_{}", log_file, since_epoch))
}

pub struct StdinHandle {
    socket: PathBuf,
    quite: bool,
}

impl StdinHandle {
    pub fn new(socket: PathBuf, quite: bool) -> Self {
        StdinHandle { socket, quite }
    }

    pub fn run(&self, name: Option<String>) -> Result<(), Box<dyn Error>> {
        let log_id = make_id_string(name)?;

        let mut stream = UnixStream::connect(&self.socket)
            .map_err(|err| format!("Error connecting to socket: {}", err))?;

        let conect_id = format!("connect -ID- {}\n", log_id);
        let content_id = format!("{} -ENDID- ", log_id);

        // make initial connection
        stream.write_all(&conect_id.as_bytes())?;

        for line in stdin().lock().lines() {
            let mut line = line.unwrap();
            line.insert_str(0, &content_id);

            // add the \n back in as lines will return without one
            line += "\n";

            stream.write_all(line.as_bytes())?;

            if !self.quite {
                println!("line: {}", line);
            }
        }

        stream.write_all(b"end\n")?;

        Ok(())
    }
}
