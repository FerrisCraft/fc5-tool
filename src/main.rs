#![allow(clippy::needless_question_mark)] // stupid lint

use camino::Utf8PathBuf;
use clap::Parser;
use eyre::Error;
use tracing_subscriber::layer::SubscriberExt;

use crate::data::World;

mod command;
mod data;

#[derive(Debug, clap::Parser)]
struct Args {
    /// Path to world directory
    world: Utf8PathBuf,
    #[arg(long, short)]
    debug: bool,
    #[command(subcommand)]
    command: command::Command,
}

#[culpa::throws]
fn main() {
    color_eyre::install()?;

    let args = Args::parse();

    tracing::subscriber::set_global_default({
        let mut fmt = tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .compact();
        if args.debug {
            fmt = fmt.with_span_events(tracing_subscriber::fmt::format::FmtSpan::ENTER);
        }
        fmt.finish().with(tracing_error::ErrorLayer::default())
    })?;

    let world = World::new(args.world);

    args.command.run(world)?;
}
