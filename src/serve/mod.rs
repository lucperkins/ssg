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
use log::trace;
use notify::Event;
use ws::{Message, Sender as WsSender, WebSocket};

use self::watch::async_watch;

mod watch;

#[derive(Debug, thiserror::Error)]
pub enum ServeError {
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

#[allow(dead_code)]
async fn handle_request(_: Request<Body>, _: PathBuf) -> Result<Response<Body>, ServeError> {
    Err(ServeError::Unknown)
}

#[allow(dead_code)]
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

pub trait Buildable {
    fn build(&self) -> Result<(), Box<ServeError>>;
}

// The main serving function
pub async fn serve(
    root: &Path,
    poll_interval: Duration,
    watchables: Vec<&str>,
    open: bool,
    port: u16,
    live_reload_port: u16,
    buildable: Box<&dyn Buildable>,
) -> Result<(), Box<ServeError>> {
    trace!("building site");

    buildable.build()?;

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    trace!("opening channel");

    let (_, rx): (Sender<Event>, Receiver<Event>) = channel();

    trace!("registering watchables");

    for entry in watchables {
        let watch_path = root.join(entry);
        trace!("listening on {:?}", watch_path);
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("could not build tokio runtime");

            runtime.block_on(async {
                async_watch(&watch_path, poll_interval)
                    .await
                    .unwrap_or_else(|_| panic!("could not watch {:?}", &watch_path));
            });
        });
    }

    trace!("listening to all directories");

    let _broadcaster: WsSender = {
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
                    let address = format!("http://{addr}");
                    open::that(address).expect("could not open browser"); // TODO: handle this better
                }

                server.await.expect("couldn't start web server"); // TODO: handle this better
            });
        });

        // TODO: make the handler callback more robust
        let ws_server = WebSocket::new(|_: WsSender| move |_: Message| Ok(()))
            .map_err(ServeError::WebSocket)?;

        let ws_addr = format!("http://localhost:{live_reload_port}");

        let ws_server = ws_server.bind(&*ws_addr).expect("could not bind WS server");

        ws_server.broadcaster()
    };

    ctrlc::set_handler(move || {
        std::process::exit(0);
    })
    .expect("error applying ctrl+c handler");

    trace!("listening...");

    while let Ok(event) = rx.recv() {
        use notify::EventKind::*;

        match event.kind {
            Create(_) | Modify(_) | Remove(_) => {
                if let Some(path) = event.paths.get(0) {
                    buildable.build()?;
                    trace!("tracked event: {:?}", path);
                }
            }
            _ => {
                trace!("untracked event: {:?}", event);
                continue;
            }
        }
    }

    Ok(())
}
