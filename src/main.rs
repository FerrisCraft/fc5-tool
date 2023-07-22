#![allow(clippy::needless_question_mark)] // stupid lint

use clap::Parser;
use eyre::Error;
use tracing_subscriber::layer::SubscriberExt;

mod app;
mod config;
mod data;

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(long, short)]
    debug: bool,

    #[command(flatten)]
    app: app::App,
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

    args.app.run()?;
}
