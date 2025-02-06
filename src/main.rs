mod answer;
mod header;
mod parser;
mod question;
mod server;

use std::net::Ipv4Addr;
use clap::Parser as CliParser;
use log::{info, log_enabled, Level, LevelFilter};
use answer::{Answer, RData};
use bytes::{Bytes, BytesMut};
use futures::{SinkExt, StreamExt};
use header::Header;
use parser::{Parser, UdpPacket};
use question::{Question, QuestionClass, QuestionType};
use server::DnsServer;
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

#[derive(CliParser)]
#[command(version)]
struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    addr: String,

    #[arg(long, default_value_t = 2053)]
    port: u16,

    #[arg(long)]
    resolver: Option<String>,

    #[arg(short, long, default_value = "info")]
    loglevel: String,
}

fn setup_logger(log_level: LevelFilter) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)))
        .level(log_level)
        .chain(std::io::stdout())
        // .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let log_level = match args.loglevel.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => {
            println!("Invalid log level '{}', defaulting to 'info'", args.loglevel);
            LevelFilter::Info
        }
    };

    setup_logger(log_level).unwrap();

    let server = DnsServer::new(
        args.addr,
        args.port,
        args.resolver,
    );

    server.run().await
}
