use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use axum::{
    body::Body,
    http::{Request, Response},
};

use notify::{Event, RecursiveMode};

use self::watch::async_watch;

mod watch;

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

async fn handle_request(_: Request<Body>, _: PathBuf) -> Result<Response<Body>, ServeError> {
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

// The main serving function
async fn serve(
    root: &Path,
    config_path: &str,
    content_dir: &str,
    poll_interval: Duration,
    watchables: Vec<(&str, WatchMode)>,
    open: bool,
    live_reload_port: u16,
) -> Result<(), ServeError> {
    build_site()?;

    let (_tx, rx): (Sender<Event>, Receiver<Event>) = channel();

    for (entry, mode) in watchables {
        let watch_path = root.join(entry);
        let should_watch = match mode {
            WatchMode::Required => true,
            WatchMode::Optional => watch_path.exists(),
            WatchMode::Condition(b) => b && watch_path.exists(),
        };
        if should_watch {
            async_watch(watch_path, poll_interval).await?;
        }
    }

    // TODO: open the browser to the running site
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
