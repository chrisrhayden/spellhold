use std::{fs, thread};
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver};
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::os::unix::net::{UnixListener, UnixStream};

use crate::SendEvt;

type ClientTuple = (Arc<AtomicBool>, mpsc::Sender<SendEvt>);

type ArcMutexReceiver = Arc<Mutex<mpsc::Receiver<SendEvt>>>;

pub struct SocketHandler {
    pub receiver: Receiver<SendEvt>,
    pub client_accept: Option<Arc<AtomicBool>>,
    pub client_sender: Option<mpsc::Sender<SendEvt>>,
}

impl SocketHandler {
    /// will spawn a main thread and wait for a connection
    /// when a connection is accepted
    /// another thread will spawn to listen on or send to
    pub fn new(socket_path: &Arc<PathBuf>) -> Self {
        let (main_sender, main_receiver) = mpsc::channel();
        let (client_sender, client_receiver) = mpsc::channel();

        let client_accept = Arc::new(AtomicBool::new(false));
        let client_receiver = Arc::new(Mutex::new(client_receiver));

        // TODO: get rid of these clones for lifetimes right?
        let main_sender_give = main_sender.clone();
        let socket_path_give = socket_path.clone();
        let client_accept_give = client_accept.clone();

        // spawn the main receiver thread
        thread::spawn(move || {
            if let Err(err) = stream_handler(
                &socket_path_give,
                &main_sender_give,
                &client_accept_give,
                &client_receiver,
            ) {
                eprintln!("Error in the main receiver thread: {}", err);
            };
        });

        SocketHandler {
            receiver: main_receiver,
            client_sender: Some(client_sender),
            client_accept: Some(client_accept),
        }
    }

    /// return the client handles replacing them with None
    pub fn get_client_handles(
        &mut self,
    ) -> Result<ClientTuple, Box<dyn Error>> {
        let abool = self.client_accept.take().ok_or("no client atomic bool")?;

        let sender = self.client_sender.take().ok_or("no mpsc::Sender")?;

        Ok((abool, sender))
    }
}

impl Iterator for SocketHandler {
    type Item = SendEvt;

    fn next(&mut self) -> Option<SendEvt> {
        match self.receiver.recv() {
            Ok(val) => Some(val),
            Err(err) => {
                eprintln!("SocketHandler Error {}", err);
                None
            }
        }
    }
}

/// waits for a connection to the main socket
/// then spawns ether,
/// a listener to take in lines from the socket
/// or a sender to give line to a client
fn stream_handler(
    socket_path: &Arc<PathBuf>,
    main_sender: &mpsc::Sender<SendEvt>,
    client_accept: &Arc<AtomicBool>,
    client_receiver: &Arc<Mutex<mpsc::Receiver<SendEvt>>>,
) -> Result<(), Box<dyn Error>> {
    // remove old file
    if socket_path.exists() {
        fs::remove_file(socket_path.as_ref())?;
    }

    // make a new buffer outside of the loop
    let mut buffer = String::new();

    let listener = UnixListener::bind(socket_path.as_ref())?;

    // listener / accept loop, will only end on error
    loop {
        // get the stream, blocking, ignoring the socket addr
        let (stream, _) = listener
            .accept()
            .map_err(|err| format!("Error accepting stream: {}", err))?;

        // TODO: this kinda sucks from here down
        BufReader::new(&stream)
            .read_line(&mut buffer)
            .expect("cant get initial line");

        let stream_send = stream.try_clone()?;

        // get data from a cli tool, send to main loop
        if buffer.starts_with("connect") {
            // send first connect evt
            main_sender.send(SendEvt::Connect(buffer.to_owned()))?;

            receiver_handler(stream_send, main_sender.clone());

        // send data to a client
        } else if buffer.starts_with("client") {
            // let the main thread know to start sending
            client_accept.store(true, Ordering::Relaxed);

            client_handler(stream_send, client_receiver.clone());
        }

        // clear the first line buffer
        buffer.clear();
    }
}

/// spawn a thread to retrieve data from the cli handle
/// send it to the main loop, the buffer stream will end on its own
fn receiver_handler(share_stream: UnixStream, sender: mpsc::Sender<SendEvt>) {
    thread::spawn(move || {
        for line in BufReader::new(share_stream).lines() {
            let evt = SendEvt::new(line.unwrap());

            sender.send(evt).expect("fucked sending cli event");
        }
    });
}

fn client_handler(stream: UnixStream, receiver: ArcMutexReceiver) {
    thread::spawn(move || {
        let mut stream = stream;
        let receiver = receiver.lock().expect("cant get client receiver");

        loop {
            match receiver.recv().unwrap() {
                SendEvt::SendString(mut val) => {
                    val += "\n";
                    stream
                        .write_all(val.as_bytes())
                        .expect("cant write to client");
                }
                SendEvt::Kill => break,
                _ => continue,
            };
        }
    });
}
