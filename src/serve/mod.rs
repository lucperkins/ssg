use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

use axum::Router;
use axum::{
    body::Body,
    http::{Request, Response},
};

use notify::Event;
use ws::{Message, Sender as WsSender, WebSocket};

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

    #[error("websocket error: {0}")]
    WebSocket(#[from] ws::Error),
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

fn rebuild_done_handling(broadcaster: &WsSender, res: Result<(), ServeError>, reload_path: &str) {
    match res {
        Ok(_) => broadcaster
            .send(format!(
                r#"
        {{
            "command": "reload",
            "path": {},
            "originalPath": "",
            "liveCSS": true,
            "liveImg": true,
            "protocol": ["http://livereload.com/protocols/official-7"]
        }}"#,
                serde_json::to_string(&reload_path).unwrap()
            ))
            .expect("could not broadcast upon done building"),
        Err(e) => {
            println!("error: {:?}", e);
        }
    }
}

// The main serving function
async fn serve(
    root: &Path,
    config_path: &str,
    content_dir: &str,
    poll_interval: Duration,
    watchables: Vec<(&str, WatchMode)>,
    open: bool,
    port: u16,
    live_reload_port: u16,
) -> Result<(), ServeError> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

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

    let broadcaster: WsSender = {
        thread::spawn(move || {
            // Create a new async runtime for the web server
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("could not build tokio runtime");

            runtime.block_on(async {
                let router = Router::new();

                let server = axum::Server::bind(&addr).serve(router.into_make_service());

                // TODO: open the browser to the running site
                if open {
                    let address = format!("http://{}", &addr);
                    open::that(address).expect("could not open browser"); // TODO: handle this better
                }

                server.await.expect("couldn't start web server"); // TODO: handle this better
            });
        });

        // TODO: make the handler callback more robust
        let ws_server = WebSocket::new(|_: WsSender| move |_: Message| Ok(()))?;

        let ws_addr = "";

        let ws_server = ws_server.bind(&*ws_addr).expect("could not bind WS server");

        ws_server.broadcaster()
    };

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
