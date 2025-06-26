mod cli;
mod filter;
mod handler;
mod metadata;
mod watcher;

use crate::{cli::Cli, handler::FsMessage, watcher::Watcher};
use clap::Parser;
use notify::RecursiveMode;
use std::{error::Error, path::Path, thread};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let (tx, rx) = crossbeam_channel::unbounded();
    let filter = cli::build_filter(cli.filter_optout, cli.filter_optin)?;
    let mut watcher = Watcher::create(tx, filter)?;

    let init_handle = watcher.watch(Path::new(&cli.path), RecursiveMode::Recursive)?;
    let notify_handle = thread::spawn(move || {
        for event in rx {
            match event {
                FsMessage::Event(event) => {
                    if let Ok(msg) = serde_json::to_string(&event) {
                        println!("{msg}");
                    }
                }
                FsMessage::Error(error) => {
                    if let Ok(msg) = serde_json::to_string(&error) {
                        eprintln!("{msg}");
                    }
                }
            }
        }
    });

    init_handle.join().expect("init thread failed")?;
    notify_handle.join().expect("notify thread failed");

    Ok(())
}
