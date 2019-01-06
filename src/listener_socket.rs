use std::{fs, thread};
use std::error::Error;
use std::path::PathBuf;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::SendEvt;

pub struct Listener {
    pub receiver: Receiver<SendEvt>,
    pub client_accept: Arc<AtomicBool>,
    pub client_sender: mpsc::Sender<String>,
}

impl Listener {
    pub fn new(socket_path: &PathBuf) -> Listener {
        let (tx, rx) = mpsc::channel();

        let socket_path1 = socket_path.clone();

        let tx1 = tx.clone();

        let client_accept = Arc::new(AtomicBool::new(false));

        let (client_sender, clieant_receiver): (
            mpsc::Sender<String>,
            mpsc::Receiver<String>,
        ) = mpsc::channel();

        let client_accept1 = client_accept.clone();

        thread::spawn(move || {
            if let Err(err) = stream_handler(
                &socket_path1,
                tx1,
                client_accept1,
                clieant_receiver,
            ) {
                eprintln!("Listener thread Error: {}", err);
            };
        });

        Listener {
            receiver: rx,
            client_sender,
            client_accept,
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

fn stream_handler(
    socket_path: &PathBuf,
    sender: mpsc::Sender<SendEvt>,
    client_accept: Arc<AtomicBool>,
    clieant_receiver: mpsc::Receiver<String>,
) -> Result<(), Box<dyn Error>> {
    if socket_path.exists() {
        fs::remove_file(socket_path).unwrap();
    }

    let socket_path1 = socket_path.clone();

    thread::spawn(move || {
        // get a handle of a socket
        let listener = match UnixListener::bind(&socket_path1) {
            Ok(listener) => listener,
            Err(err) => {
                let path = socket_path1.to_string_lossy();
                panic!("couldn't connect to socket_path {} {}", path, err);
            }
        };

        // TODO: make a brake point
        loop {
            let stream = match listener.accept() {
                Ok((stream, _)) => stream,
                Err(err) => {
                    panic!("thread error: {}", err);
                }
            };

            let share_stream =
                stream.try_clone().expect("cant clone stream for thread");

            let sender1 = sender.clone();

            let mut bf_stream = BufReader::new(&stream);

            let mut buffer = String::new();

            bf_stream
                .read_line(&mut buffer)
                .expect("cant get initial line");

            if buffer.starts_with("connect") {
                sender
                    .send(SendEvt::Connect(buffer))
                    .expect("failed to send connect event");

                recever_handler(share_stream, sender1);
            } else if buffer.starts_with("client") {
                client_accept.store(true, Ordering::Relaxed);

                let mut new_stream = stream.try_clone();
                let mut new_recever = Arc::new(clieant_receiver);

                thread::spawn(move || {
                    let new_stream = new_stream.unwrap();
                    for next in clieant_receiver.recv() {
                        new_stream.write_all(next.as_bytes()).unwrap();
                    }
                });
            }
        }
    });

    Ok(())
}

fn recever_handler(share_stream: UnixStream, sender: mpsc::Sender<SendEvt>) {
    thread::spawn(move || {
        let bf_stream = BufReader::new(share_stream);

        for line in bf_stream.lines() {
            let line = line.expect("bad utf8?");

            let evt = evt_dispatch(line);

            if let SendEvt::Kill = evt {
                sender.send(evt).expect("fucked sending");
                break;
            }

            sender.send(evt).expect("fucked sending");
        }
    });
}

fn client_handler() {}

fn evt_dispatch(line: String) -> SendEvt {
    if line.starts_with("kill") {
        SendEvt::Kill
    } else if line.starts_with("end") {
        SendEvt::None
    } else if !line.is_empty() {
        SendEvt::SendString(line)
    } else {
        // idk how well get here
        SendEvt::None
    }
}
