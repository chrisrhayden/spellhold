use std::fs;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::io::Write;

pub struct Client {
    pub sender: Sender<String>,
    pub client_accept: Arc<AtomicBool>,
}

impl Client {
    pub fn new(socket_path: &PathBuf) -> Client {
        let (sender, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

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

        let client_accept = Arc::new(AtomicBool::new(false));

        let client_accept1 = Arc::clone(&client_accept);

        thread::spawn(move || loop {
            match listener.accept() {
                Ok((mut iner_stream, _)) => {
                    // set the sender gate bool to true
                    client_accept1.store(true, Ordering::Relaxed);

                    loop {
                        let next = match rx.recv() {
                            Err(err) => {
                                eprintln!("Client Error: {}", err);
                                break;
                            }
                            Ok(val) => val,
                        };

                        // send line to the client
                        iner_stream.write_all(next.as_bytes()).unwrap();
                    }

                    client_accept1.store(false, Ordering::Relaxed);
                }
                Err(err) => eprintln!("Error: {}", err),
            }
        });

        Client {
            sender,
            client_accept,
        }
    }
}
