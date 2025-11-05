use anyhow::{Context, Result};
use clap::Parser;
use tokio::io::AsyncReadExt;
use tokio_serial::{DataBits, Parity, SerialPortBuilderExt, StopBits};
use tracing::{error, info, instrument};

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

#[derive(Parser, Debug)]
#[command(name = "serial-monitor")]
#[command(about = "Monitor multiple serial ports and print received data", long_about = None)]
struct Args {
    /// Serial ports to monitor (e.g., /dev/ttyUSB0 /dev/ttyUSB1)
    #[arg(short = 'p', required = true)]
    ports: Vec<String>,

    /// Baud rate
    #[arg(short='b', long, default_value = "9600")]
    baud_rate: u32,

    /// Data bits (5, 6, 7, or 8)
    #[arg(short='d', long, default_value = "8")]
    data_bits: DataBitsArg,

    /// Parity (none, odd, or even)
    #[arg(short='r', long, default_value = "none")]
    parity: ParityArg,

    /// Stop bits (1 or 2)
    #[arg(short='s', long, default_value = "1")]
    stop_bits: StopBitsArg,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum ParityArg {
    None,
    Odd,
    Even,
}

impl From<ParityArg> for Parity {
    fn from(parity: ParityArg) -> Self {
        match parity {
            ParityArg::None => Parity::None,
            ParityArg::Odd => Parity::Odd,
            ParityArg::Even => Parity::Even,
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum DataBitsArg {
    #[value(name = "5")]
    Five,
    #[value(name = "6")]
    Six,
    #[value(name = "7")]
    Seven,
    #[value(name = "8")]
    Eight,
}

impl From<DataBitsArg> for DataBits {
    fn from(data_bits: DataBitsArg) -> Self {
        match data_bits {
            DataBitsArg::Five => DataBits::Five,
            DataBitsArg::Six => DataBits::Six,
            DataBitsArg::Seven => DataBits::Seven,
            DataBitsArg::Eight => DataBits::Eight,
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum StopBitsArg {
    #[value(name = "1")]
    One,
    #[value(name = "2")]
    Two,
}

impl From<StopBitsArg> for StopBits {
    fn from(stop_bits: StopBitsArg) -> Self {
        match stop_bits {
            StopBitsArg::One => StopBits::One,
            StopBitsArg::Two => StopBits::Two,
        }
    }
}

#[instrument(skip(data_bits, parity, stop_bits))]
async fn monitor_port(
    port_name: String,
    baud_rate: u32,
    data_bits: DataBits,
    parity: Parity,
    stop_bits: StopBits,
) -> Result<()> {
    info!("Opening port...");

    let mut port = tokio_serial::new(&port_name, baud_rate)
        .data_bits(data_bits)
        .parity(parity)
        .stop_bits(stop_bits)
        .open_native_async()
        .with_context(|| format!("Failed to open port {}", port_name))?;

    let config_str = format!(
        "{}bps {}{}{}",
        baud_rate,
        match data_bits {
            DataBits::Five => "5",
            DataBits::Six => "6",
            DataBits::Seven => "7",
            DataBits::Eight => "8",
        },
        match parity {
            Parity::None => "N",
            Parity::Odd => "O",
            Parity::Even => "E",
        },
        match stop_bits {
            StopBits::One => "1",
            StopBits::Two => "2",
        }
    );
    info!("Monitoring ({})", config_str);

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

    // Convert arguments
    let data_bits: DataBits = args.data_bits.into();
    let stop_bits: StopBits = args.stop_bits.into();
    let parity: Parity = args.parity.into();

    info!(
        "Starting serial monitor for {} port(s)...",
        args.ports.len()
    );

    // Spawn a task for each serial port
    let mut tasks = Vec::new();
    for port in args.ports {
        let task = tokio::spawn(monitor_port(
            port.clone(),
            args.baud_rate,
            data_bits,
            parity,
            stop_bits,
        ));
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
