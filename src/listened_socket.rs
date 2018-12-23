use std::fs;
use std::thread;
use std::path::PathBuf;
use std::os::unix::net::UnixListener;
use std::sync::mpsc::{self, Receiver};
use std::io::Read;

pub enum SendEvt {
    Kill,
    SendString(String),
    None,
}

pub struct Listener {
    pub receiver: Receiver<SendEvt>,
}

impl Listener {
    pub fn new(socket_path: PathBuf) -> Listener {
        let (tx, rx) = mpsc::channel();

        if socket_path.exists() {
            fs::remove_file(&socket_path).unwrap();
        }

        // get a handle of a socket
        let stream = match UnixListener::bind(&socket_path) {
            Ok(stream) => stream,
            Err(_) => {
                let path = socket_path.to_string_lossy();
                panic!("couldn't connect to socket_path {}", path);
            }
        };

        // make a clone to send to the thread
        let stream1 = match stream.try_clone() {
            Err(err) => panic!("couldn't clone stream: {}", err),
            Ok(stream) => stream,
        };

        let tx1 = tx.clone();

        thread::spawn(move || {
            for client in stream1.incoming() {
                let mut client = client.unwrap();

                let mut buf_string = String::new();

                client
                    .read_to_string(&mut buf_string)
                    .expect("received string with invalid utf8 characters");

                let buf_string = buf_string.trim_start().trim_end();

                tx1.send(SendEvt::SendString(buf_string.to_string()))
                    .expect("couldn't send event");
            }

            tx1.send(SendEvt::Kill).unwrap();
        });

        Listener { receiver: rx }
    }
}
