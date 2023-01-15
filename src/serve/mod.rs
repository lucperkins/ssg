use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

use axum::{
    body::Body,
    http::{Request, Response},
};

use notify::{Event, RecursiveMode, Watcher};

const RECURSIVE: RecursiveMode = RecursiveMode::Recursive;

#[derive(Debug, thiserror::Error)]
enum ServeError {
    #[error("unknown")]
    Unknown,

    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("i/o error: {0}")]
    IO(#[from] std::io::Error),

    #[error("notify error: {0}")]
    Notify(#[from] notify::Error),
}

const LIVE_RELOAD: &str = include_str!("livereload.js");

async fn handle_request(req: Request<Body>, root: PathBuf) -> Result<Response<Body>, ServeError> {
    Err(ServeError::Unknown)
}

// Build the site to "warm up" the server
fn build_site() -> Result<(), ServeError> {
    // TODO
    Ok(())
}

enum WatchMode {
    Required,
    Optional,
    Condition(bool),
}

fn handle_watch_event(res: Result<Event, notify::Error>) {
    match res {
        Ok(event) => println!("event: {:?}", event),
        Err(e) => println!("watch error: {:?}", e),
    }
}

async fn serve(
    root: &Path,
    config_path: &str,
    content_dir: &str,
    open: bool,
) -> Result<(), ServeError> {
    build_site()?;

    let (_tx, rx): (Sender<Event>, Receiver<Event>) = channel();

    let watch_this: Vec<(&str, WatchMode)> = vec![
        (config_path, WatchMode::Required),
        (content_dir, WatchMode::Required),
    ];

    for (entry, mode) in watch_this {
        let watch_path = root.join(entry);
        let should_watch = match mode {
            WatchMode::Required => true,
            WatchMode::Optional => watch_path.exists(),
            WatchMode::Condition(b) => b && watch_path.exists(),
        };
        if should_watch {
            let mut watcher = notify::recommended_watcher(handle_watch_event)?;
            watcher.watch(&root.join(entry), RECURSIVE)?;
        }
    }

    if open {
        open::that("")?;
    }

    while let Ok(event) = rx.recv() {
        use notify::EventKind::*;

        println!("attrs: {:?}", event.attrs);

        match event.kind {
            Create(create) => {
                println!("creating: {:?}", create);
            }
            Modify(modify) => {
                println!("modifying: {:?}", modify);
            }
            Remove(remove) => {
                println!("removing: {:?}", remove);
            }
            _ => continue,
        }
    }

    Ok(())
}
