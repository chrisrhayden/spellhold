use std::fs;
use std::thread;
use std::io::Read;
use std::path::PathBuf;
use std::os::unix::net::UnixListener;

use std::sync::mpsc::{self, Receiver};

use crate::SendEvt;

pub struct Listener {
    pub receiver: Receiver<SendEvt>,
    pub listener: UnixListener,
}

impl Listener {
    pub fn new(socket_path: &PathBuf) -> Listener {
        let (tx, rx) = mpsc::channel();

        if socket_path.exists() {
            fs::remove_file(&socket_path).unwrap();
        }

        // get a handle of a socket
        let listener = match UnixListener::bind(&socket_path) {
            Ok(listener) => listener,
            Err(err) => {
                let path = socket_path.to_string_lossy();
                panic!("couldn't connect to socket_path {} {}", path, err);
            }
        };

        // make a clone to send to the thread
        let listener1 = match listener.try_clone() {
            Err(err) => panic!("couldn't clone listener: {}", err),
            Ok(listener) => listener,
        };

        let tx1 = tx.clone();

        thread::spawn(move || {
            for client in listener1.incoming() {
                let mut client = client.unwrap();

                let mut buf_string = String::new();

                while buf_string.ends_with() != "\n" {
                    client
                        .read_to_string(&mut buf_string)
                        .expect("received string with invalid utf8 characters");
                }

                let buf_string = buf_string.trim_start().trim_end();

                tx1.send(SendEvt::SendString(buf_string.to_string()))
                    .expect("couldn't send event");
            }

            tx1.send(SendEvt::Kill).unwrap();
        });

        Listener {
            receiver: rx,
            listener,
        }
    }
}

impl Iterator for Listener {
    type Item = SendEvt;

    fn next(&mut self) -> Option<SendEvt> {
        match self.receiver.recv() {
            Ok(val) => Some(val),
            Err(err) => Some(SendEvt::Err(err.to_string())),
        }
    }
}
