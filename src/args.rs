use clap::Parser;
use tokio_serial::{DataBits, Parity, StopBits};

#[derive(Parser, Debug)]
#[command(name = "serial-monitor")]
#[command(about = "Monitor multiple serial ports and print received data", long_about = None)]
pub struct Args {
    /// Serial ports to monitor (e.g., /dev/ttyUSB0 /dev/ttyUSB1)
    #[arg(short = 'p', required = true)]
    pub ports: Vec<String>,

    /// Baud rate
    #[arg(short = 'b', long, default_value = "9600")]
    pub baud_rate: u32,

    /// Data bits (5, 6, 7, or 8)
    #[arg(short = 'd', long, default_value = "8")]
    pub data_bits: DataBitsArg,

    /// Parity (none, odd, or even)
    #[arg(short = 'r', long, default_value = "none")]
    pub parity: ParityArg,

    /// Stop bits (1 or 2)
    #[arg(short = 's', long, default_value = "1")]
    pub stop_bits: StopBitsArg,
}

impl Args {
    pub(crate) fn serial(&self) -> SerialConfig {
        SerialConfig {
            baud_rate: self.baud_rate,
            data_bits: self.data_bits.into(),
            parity: self.parity.into(),
            stop_bits: self.stop_bits.into(),
        }
    }
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum ParityArg {
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
pub enum DataBitsArg {
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
pub enum StopBitsArg {
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct SerialConfig {
    pub(crate) baud_rate: u32,
    pub(crate) data_bits: DataBits,
    pub(crate) parity: Parity,
    pub(crate) stop_bits: StopBits,
}

impl std::fmt::Display for SerialConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}bps {}/{}/{}",
            self.baud_rate,
            match self.data_bits {
                DataBits::Five => "5",
                DataBits::Six => "6",
                DataBits::Seven => "7",
                DataBits::Eight => "8",
            },
            match self.parity {
                Parity::None => "N",
                Parity::Odd => "O",
                Parity::Even => "E",
            },
            match self.stop_bits {
                StopBits::One => "1",
                StopBits::Two => "2",
            }
        )
    }
}