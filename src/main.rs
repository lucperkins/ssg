use std::{path::Path, time::Duration};

use ssg::{serve, Buildable, ServeError};

extern crate ssg;

struct Site;

impl Buildable for Site {
    fn build(&self) -> Result<(), Box<ServeError>> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<ServeError>> {
    env_logger::init();

    let root = Path::new("site");

    let site = Site;

    serve(
        root,
        Duration::from_secs(5),
        vec!["docs"],
        false,
        8080,
        8081,
        Box::new(&site),
    )
    .await?;

    Ok(())
}
