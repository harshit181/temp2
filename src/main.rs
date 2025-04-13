//! Main executable for Trafilatura Rust port.

fn main() {
    if let Err(e) = trafilatura::cli::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}