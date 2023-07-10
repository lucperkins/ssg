extern crate ssg;

use clap::Parser;
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

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<ServeError>> {
    let Cli { port } = Cli::parse();

    let mut config = ServeConfig::default();
    config.port = port;

    let site = Site::new();

    serve(&config, Box::new(&site)).await
}
