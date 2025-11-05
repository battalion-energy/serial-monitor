mod args;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use tokio::io::AsyncReadExt;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tracing::Instrument;

struct HexBytes<'a>(&'a [u8]);

impl<'a> std::fmt::Display for HexBytes<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, byte) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

async fn monitor_port(mut port: SerialStream) -> Result<()> {
    let mut buffer = [0u8; 1024];
    loop {
        match port.read(&mut buffer).await {
            Ok(n) if n > 0 => {
                let data = &buffer[..n];
                tracing::info!("{}", HexBytes(data));
            }
            Ok(_) => {
                // No data read, continue
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout, continue
            }
            Err(e) => {
                tracing::error!("Error reading: {}", e);
                return Err(e.into());
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let args = Args::parse();

    let config = args.serial();

    // Open all ports sequentially
    let mut unspawned_tasks = Vec::new();

    for port_name in &args.ports {
        let _span = tracing::info_span!("opening", port = ?port_name).entered();
        let port = tokio_serial::new(port_name, config.baud_rate)
            .data_bits(config.data_bits)
            .parity(config.parity)
            .stop_bits(config.stop_bits)
            .open_native_async()
            .context("opening port")?;

        tracing::info!("opened port");
        let task =
            async { monitor_port(port).await }.instrument(tracing::info_span!("monitor", port=?port_name));
        unspawned_tasks.push(task);
    }

    for task in unspawned_tasks {
        tokio::spawn(task);
    }

    tracing::info!("press ctrl-c to exit");
    tokio::signal::ctrl_c().await?;
    tracing::info!("shutting down...");

    Ok(())
}
