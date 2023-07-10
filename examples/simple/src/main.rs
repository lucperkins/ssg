extern crate ssg;

use ssg::{Buildable, ServeConfig, ServeError, serve};

struct Site;

impl Site {
    fn new() -> Self { Self {} }
}

impl Buildable for Site {
    fn build(&self) -> Result<(), Box<ServeError>> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<ServeError>> {
    let config = ServeConfig::default();

    let site = Site::new();

    foo();

    serve(&config, Box::new(&site)).await
}
