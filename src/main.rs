mod args;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use tokio::io::AsyncReadExt;
use tokio_serial::SerialPortBuilderExt;
use tracing::{error, info, instrument};
use crate::args::SerialConfig;

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



#[instrument(skip(config))]
async fn monitor_port(port_name: String, config: SerialConfig) -> Result<()> {
    info!("Opening port...");

    let mut port = tokio_serial::new(&port_name, config.baud_rate)
        .data_bits(config.data_bits)
        .parity(config.parity)
        .stop_bits(config.stop_bits)
        .open_native_async()
        .with_context(|| format!("Failed to open port {}", port_name))?;

    info!("Monitoring ({})", config);

    let mut buffer = [0u8; 1024];
    loop {
        match port.read(&mut buffer).await {
            Ok(n) if n > 0 => {
                let data = &buffer[..n];
                info!("{}", HexBytes(data));
            }
            Ok(_) => {
                // No data read, continue
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout, continue
            }
            Err(e) => {
                error!("Error reading: {}", e);
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


    info!(
        "Starting serial monitor for {} port(s)...",
        args.ports.len()
    );

    // Spawn a task for each serial port
    let mut tasks = Vec::new();
    for port in args.ports {
        let task = tokio::spawn(monitor_port(port.clone(), config));
        tasks.push(task);
    }

    // Wait for all tasks (they run indefinitely unless there's an error)
    for task in tasks {
        if let Err(e) = task.await? {
            error!("Task error: {}", e);
        }
    }

    Ok(())
}
