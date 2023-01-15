use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::Path, time::Duration};

const RECURSIVE: RecursiveMode = RecursiveMode::Recursive;

async fn handle_result(
    res: Result<Event, notify::Error>,
    tx: &mut Sender<Result<Event, notify::Error>>,
) {
    // TODO: refine this logic
    tx.send(res).await.unwrap();
}

fn async_watcher(
    poll_interval: Duration,
) -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let config = Config::default().with_poll_interval(poll_interval);

    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                handle_result(res, &mut tx).await;
            })
        },
        config,
    )?;

    Ok((watcher, rx))
}

pub(crate) async fn async_watch<P: AsRef<Path>>(
    path: P,
    poll_interval: Duration,
) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher(poll_interval)?;

    watcher.watch(path.as_ref(), RECURSIVE)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(_event) => {
                //println!("changed: {:?}", event)
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
