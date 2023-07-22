#![allow(clippy::needless_question_mark)] // stupid lint

use clap::Parser;
use eyre::Error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app;
mod config;
mod data;

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(long, short, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(long, short, action = clap::ArgAction::Count)]
    quiet: u8,

    #[arg(long)]
    trace: bool,

    #[command(flatten)]
    app: app::App,
}

#[culpa::throws]
fn main() {
    color_eyre::install()?;

    let args = Args::parse();

    tracing_subscriber::registry()
        .with({
            let mut fmt = tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .compact();
            if args.trace {
                fmt = fmt.with_span_events(tracing_subscriber::fmt::format::FmtSpan::ENTER);
            }
            fmt
        })
        .with(tracing_error::ErrorLayer::default())
        .with(match args.verbose as i16 - args.quiet as i16 {
            i16::MIN..=-3 => tracing_subscriber::filter::LevelFilter::ERROR,
            -2 => tracing_subscriber::filter::LevelFilter::ERROR,
            -1 => tracing_subscriber::filter::LevelFilter::WARN,
            0 => tracing_subscriber::filter::LevelFilter::INFO,
            1 => tracing_subscriber::filter::LevelFilter::DEBUG,
            2..=i16::MAX => tracing_subscriber::filter::LevelFilter::TRACE,
        })
        .init();

    args.app.run()?;
}
