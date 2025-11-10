# Serial Monitor

A Rust-based command-line utility for monitoring multiple serial ports simultaneously. Ideal for identifying which ports are actively receiving data and debugging serial communications.

## Use Case

When working with multiple serial devices, it's often unclear which port is receiving data. This tool allows you to monitor several ports at once and see in real-time which ones are active and what data they're receiving.

## Features

- **Multi-port monitoring** - Monitor multiple serial ports concurrently
- **Hex output** - Displays received data in space-separated hexadecimal format
- **Real-time logging** - See which ports are receiving data as it arrives
- **Configurable serial settings** - Support for various baud rates, data bits, parity, and stop bits
- **Graceful shutdown** - Press Ctrl+C to exit cleanly

## Installation

Build from source using Cargo:

```bash
cargo build --release
```

The binary will be available at `target/release/serial-monitor`.

## Usage

### Basic Usage

Monitor a single port with default settings (9600 baud, 8N1):

```bash
serial-monitor -p /dev/ttyUSB0
```

### Monitor Multiple Ports

Monitor several ports to see which ones are active:

```bash
serial-monitor -p /dev/ttyUSB0 -p /dev/ttyUSB1 -p /dev/ttyACM0
```

### Custom Serial Configuration

Specify baud rate and serial parameters:

```bash
serial-monitor -p /dev/ttyUSB0 -b 115200 -d 8 -r none -s 1
```

## Command-Line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--port` | `-p` | Serial port to monitor (can be specified multiple times) | Required |
| `--baud-rate` | `-b` | Baud rate in bits per second | 9600 |
| `--data-bits` | `-d` | Number of data bits (5, 6, 7, 8) | 8 |
| `--parity` | `-r` | Parity checking (none, odd, even) | none |
| `--stop-bits` | `-s` | Number of stop bits (1, 2) | 1 |

## Example Output

```
2025-11-10T12:34:56.789Z INFO serial_monitor: Opening port "/dev/ttyUSB0" with config: 9600 bps 8/N/1
2025-11-10T12:34:56.790Z INFO serial_monitor: Opening port "/dev/ttyUSB1" with config: 9600 bps 8/N/1
2025-11-10T12:34:57.123Z INFO monitor{port="/dev/ttyUSB0"}: serial_monitor: 48 65 6C 6C 6F
2025-11-10T12:34:58.456Z INFO monitor{port="/dev/ttyUSB0"}: serial_monitor: 57 6F 72 6C 64
```

In this example, only `/dev/ttyUSB0` is receiving data, while `/dev/ttyUSB1` remains silent.

## Common Port Names

- **Linux**: `/dev/ttyUSB0`, `/dev/ttyACM0`, `/dev/ttyS0`
- **Windows**: `COM1`, `COM2`, `COM3`, etc.

## Exit

Press `Ctrl+C` to stop monitoring and exit the application.

## License

MIT License - Copyright (c) Battalion Energy Inc

## Requirements

- Rust 2024 edition or later
- Access permissions to the serial ports (may require adding user to `dialout` group on Linux)

## Troubleshooting

**Permission denied**: On Linux, add your user to the dialout group:
```bash
sudo usermod -a -G dialout $USER
```
Then log out and back in for the changes to take effect.

**Port not found**: Verify the port exists:
```bash
ls -l /dev/tty*
```
