use anyhow::{Context, Result};
use clap::Parser;
use tokio::io::AsyncReadExt;
use tokio_serial::{DataBits, Parity, SerialPortBuilderExt, StopBits};

#[derive(Parser, Debug)]
#[command(name = "serial-monitor")]
#[command(about = "Monitor multiple serial ports and print received data", long_about = None)]
struct Args {
    /// Serial ports to monitor (e.g., /dev/ttyUSB0 /dev/ttyUSB1)
    #[arg(required = true)]
    ports: Vec<String>,

    /// Baud rate
    #[arg(short, long, default_value = "9600")]
    baud_rate: u32,

    /// Data bits (5, 6, 7, or 8)
    #[arg(short, long, default_value = "8")]
    data_bits: u8,

    /// Parity (none, odd, or even)
    #[arg(short, long, default_value = "none")]
    parity: ParityArg,

    /// Stop bits (1 or 2)
    #[arg(short, long, default_value = "1")]
    stop_bits: u8,
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

async fn monitor_port(
    port_name: String,
    baud_rate: u32,
    data_bits: DataBits,
    parity: Parity,
    stop_bits: StopBits,
) -> Result<()> {
    println!("[{}] Opening port...", port_name);

    let mut port = tokio_serial::new(&port_name, baud_rate)
        .data_bits(data_bits)
        .parity(parity)
        .stop_bits(stop_bits)
        .open_native_async()
        .with_context(|| format!("Failed to open port {}", port_name))?;

    println!(
        "[{}] Monitoring ({}bps {}{}{})",
        port_name,
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

    let mut buffer = [0u8; 1024];
    loop {
        match port.read(&mut buffer).await {
            Ok(n) if n > 0 => {
                let data = &buffer[..n];
                print!("[{}] ", port_name);
                for byte in data {
                    if byte.is_ascii_graphic() || *byte == b' ' {
                        print!("{}", *byte as char);
                    } else if *byte == b'\r' {
                        print!("\\r");
                    } else if *byte == b'\n' {
                        print!("\\n");
                    } else if *byte == b'\t' {
                        print!("\\t");
                    } else {
                        print!("\\x{:02x}", byte);
                    }
                }
                println!();
            }
            Ok(_) => {
                // No data read, continue
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout, continue
            }
            Err(e) => {
                eprintln!("[{}] Error reading: {}", port_name, e);
                return Err(e.into());
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Validate and convert arguments
    let data_bits = match args.data_bits {
        5 => DataBits::Five,
        6 => DataBits::Six,
        7 => DataBits::Seven,
        8 => DataBits::Eight,
        _ => {
            eprintln!("Invalid data bits: {}. Must be 5, 6, 7, or 8", args.data_bits);
            std::process::exit(1);
        }
    };

    let stop_bits = match args.stop_bits {
        1 => StopBits::One,
        2 => StopBits::Two,
        _ => {
            eprintln!("Invalid stop bits: {}. Must be 1 or 2", args.stop_bits);
            std::process::exit(1);
        }
    };

    let parity: Parity = args.parity.into();

    println!(
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
            eprintln!("Task error: {}", e);
        }
    }

    Ok(())
}
