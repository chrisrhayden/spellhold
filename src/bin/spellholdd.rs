use spellhold::daemon::Daemon;

fn main() {
    let mut da = Daemon::new();

    if let Err(err) = da.run() {
        eprintln!("Error {}", err);
    }
}
