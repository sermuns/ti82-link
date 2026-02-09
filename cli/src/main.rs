#![allow(unused)]

use clap::{Parser, Subcommand};
use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use serialport::SerialPort;
use std::time::Duration;

mod tokens;

fn parse_ti82_real(bytes: &[u8]) -> Option<f64> {
    if bytes.len() != 9 {
        return None;
    }

    let sign_byte = bytes[0];
    let exponent = bytes[1] as i16 - 0x80;

    let mut mantissa: u64 = 0;
    for i in 0..7 {
        let digit_pair = bytes[2 + i];
        let high = ((digit_pair >> 4) & 0x0F) as u64;
        let low = (digit_pair & 0x0F) as u64;
        mantissa = mantissa * 100 + high * 10 + low;
    }

    // TI format: mantissa has implicit decimal after first digit
    // So 42000000000000 represents 4.2, and we multiply by 10^exponent
    let mut value = mantissa as f64 / 1e13;

    if exponent != 0 {
        value *= 10f64.powi(exponent as i32);
    }

    if sign_byte & 0x80 != 0 {
        value = -value;
    }

    Some(value)
}

#[derive(Parser)]
#[command(name = "ti82-cli")]
#[command(about = "TI-82 Calculator Link Tool", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "/dev/ttyUSB0")]
    port: String,

    #[arg(short, long, default_value = "57600")]
    baud: u32,

    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Status,

    Test,

    Ping,

    Dump {
        #[arg(value_parser = parse_var_name)]
        name: char,
    },

    Get {
        #[arg(value_parser = parse_var_name)]
        name: char,
    },

    GetProgram {
        name: String,
    },

    /// Receive/listen mode - waits for calculator to send a variable
    Receive,

    Send {
        #[arg(value_parser = parse_var_name)]
        name: char,

        #[arg(default_value = "42")]
        value: i32,
    },
}

fn parse_var_name(s: &str) -> Result<char, String> {
    if s.len() != 1 {
        return Err("Variable name must be a single character".to_string());
    }
    let c = s.chars().next().unwrap().to_ascii_uppercase();
    if !c.is_ascii_alphabetic() {
        return Err("Variable name must be a letter A-Z".to_string());
    }
    Ok(c)
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    let mut port = serialport::new(&cli.port, cli.baud)
        .timeout(Duration::from_millis(100))
        .open()
        .wrap_err_with(|| eyre!("port `{}` not found, is it connected?", &cli.port))?;

    if cli.verbose {
        eprintln!("Waiting for Arduino to boot...");
    }
    std::thread::sleep(Duration::from_millis(2500));
    port.clear(serialport::ClearBuffer::Input)?;

    if cli.verbose {
        eprintln!("Sending command...");
    }

    match cli.command {
        Commands::Status => {
            send_command(&mut port, b"S")?;
            let response = read_response(&mut port)?;
            println!("{response}");
        }
        Commands::Test => {
            send_command(&mut port, b"T")?;
            let mut buffer = String::new();
            let mut byte = [0u8; 1];
            let start = std::time::Instant::now();

            while start.elapsed() < Duration::from_secs(2) {
                if port.read_exact(&mut byte).is_ok() {
                    if byte[0] == b'\r' || byte[0] == b'\n' {
                        if !buffer.is_empty() {
                            println!("{buffer}");
                            buffer.clear();
                        }
                    } else {
                        buffer.push(byte[0] as char);
                    }
                }
            }
        }
        Commands::Ping => {
            send_command(&mut port, b"P")?;
            let mut buffer = String::new();
            let mut byte = [0u8; 1];
            let start = std::time::Instant::now();

            while start.elapsed() < Duration::from_secs(5) {
                if port.read_exact(&mut byte).is_ok() {
                    if byte[0] == b'\r' || byte[0] == b'\n' {
                        if !buffer.is_empty() {
                            println!("{buffer}");
                            buffer.clear();
                        }
                    } else {
                        buffer.push(byte[0] as char);
                    }
                }
            }
        }
        Commands::Dump { name } => {
            send_command(&mut port, b"D")?;
            port.write_all(&[name as u8])?;

            let mut buffer = String::new();
            let mut byte = [0u8; 1];
            let start = std::time::Instant::now();

            while start.elapsed() < Duration::from_secs(2) {
                if port.read_exact(&mut byte).is_ok() {
                    if byte[0] == b'\r' || byte[0] == b'\n' {
                        if !buffer.is_empty() {
                            println!("{buffer}");
                            buffer.clear();
                        }
                    } else {
                        buffer.push(byte[0] as char);
                    }
                }
            }
        }
        Commands::Get { name } => {
            send_command(&mut port, b"R")?;
            port.write_all(&[name as u8])?;

            let (response, data_bytes) = read_all_response(&mut port, cli.verbose)?;

            if !data_bytes.is_empty() {
                if let Some(value) = parse_ti82_real(&data_bytes) {
                    println!("Variable {name}: {value}");
                } else {
                    println!(
                        "Variable {}: [couldn't parse {} bytes]",
                        name,
                        data_bytes.len()
                    );
                }

                if cli.verbose {
                    print!("Raw bytes: ");
                    for byte in &data_bytes {
                        print!("{byte:02X} ");
                    }
                    println!();
                }
            }

            if cli.verbose {
                println!("\n{response}");
            }
        }
        Commands::GetProgram { name } => {
            // Send P + 8-byte program name (null-padded)
            let mut cmd = vec![b'P'];
            let name_bytes = name.as_bytes();
            let len = name_bytes.len().min(8);
            cmd.extend_from_slice(&name_bytes[..len]);
            for _ in len..8 {
                cmd.push(0);
            }
            port.write_all(&cmd)?;
            port.flush()?;

            let (response, data_bytes) = read_all_response(&mut port, cli.verbose)?;

            if !data_bytes.is_empty() {
                println!("Program '{}': {} bytes received", name, data_bytes.len());

                // Display raw hex dump
                print!("Hex dump: ");
                for (i, byte) in data_bytes.iter().enumerate() {
                    if i > 0 && i % 16 == 0 {
                        println!();
                        print!("          ");
                    }
                    print!("{byte:02X} ");
                }
                println!();

                // Display as ASCII where printable
                print!("ASCII:    ");
                for (i, byte) in data_bytes.iter().enumerate() {
                    if i > 0 && i % 16 == 0 {
                        println!();
                        print!("          ");
                    }
                    if byte.is_ascii_graphic() || *byte == b' ' {
                        print!(" {}  ", *byte as char);
                    } else {
                        print!(" .  ");
                    }
                }
                println!();
            } else {
                println!("Program '{name}': No data received");
            }

            if cli.verbose {
                println!("\n{response}");
            }
        }
        Commands::Receive => {
            // Send L command to enter listen/receive mode
            port.write_all(b"L")?;
            port.flush()?;

            println!("Listening for incoming transfer from calculator...");
            println!("On calculator: Press [2nd] [LINK] → SEND → Select variable");
            println!();

            let (response, data_bytes) = read_all_response(&mut port, cli.verbose)?;

            // Parse the response to extract variable type and name
            let mut var_type = None;
            let mut var_name = String::new();
            for line in response.lines() {
                if line.starts_with("Type: ") {
                    if let Some(hex) = line.strip_prefix("Type: ") {
                        if let Ok(type_id) = u8::from_str_radix(hex.trim(), 16) {
                            var_type = Some(type_id);
                        }
                    }
                    // Print the type line
                    println!("{line}");
                }
                if line.starts_with("Name: ") {
                    var_name = line.strip_prefix("Name: ").unwrap_or("").to_string();
                    // Print the name line
                    println!("{line}");
                }
                if line.starts_with("Size: ") {
                    // Print the size line
                    println!("{line}");
                }
            }

            if !data_bytes.is_empty() {
                // Check if it's a program (type 0x05 or 0x06)
                if matches!(var_type, Some(0x05) | Some(0x06)) {
                    println!("\n=== Program Content ===");
                    let decoder = tokens::TokenDecoder::new();
                    if let Some(decoded) = decoder.decode_program(&data_bytes) {
                        // Split by newlines for better readability
                        for line in decoded.split('\n') {
                            println!("{line}");
                        }
                    } else {
                        println!("Error: Could not decode program data");
                    }
                    println!("======================\n");
                } else {
                    // For non-program data, show raw dump
                    println!();
                }

                // Always show hex dump if verbose or non-program
                if cli.verbose || !matches!(var_type, Some(0x05) | Some(0x06)) {
                    print!("Hex dump: ");
                    for (i, byte) in data_bytes.iter().enumerate() {
                        if i > 0 && i % 16 == 0 {
                            println!();
                            print!("          ");
                        }
                        print!("{byte:02X} ");
                    }
                    println!();
                }
            } else {
                println!("No data received");
            }

            if cli.verbose {
                println!("\n=== Full Debug Output ===");
                println!("{response}");
            }
        }
        Commands::Send { name, value } => {
            // Send MR<name><4 bytes i32 little-endian> - all in one write!
            let mut cmd = Vec::new();
            cmd.extend_from_slice(b"MR");
            cmd.push(name as u8);
            cmd.extend_from_slice(&value.to_le_bytes());
            port.write_all(&cmd)?;
            port.flush()?;

            let mut buffer = String::new();
            let mut byte = [0u8; 1];
            let start = std::time::Instant::now();
            let mut last_output = std::time::Instant::now();

            while start.elapsed() < Duration::from_secs(15) {
                if port.read_exact(&mut byte).is_ok() {
                    last_output = std::time::Instant::now();
                    if byte[0] == b'\r' || byte[0] == b'\n' {
                        if !buffer.is_empty() {
                            println!("{buffer}");
                            if buffer.contains("OK") || buffer.contains("ERROR") {
                                break;
                            }
                            buffer.clear();
                        }
                    } else {
                        buffer.push(byte[0] as char);
                    }
                } else if last_output.elapsed() > Duration::from_secs(2) && !buffer.is_empty() {
                    // Print partial buffer if no output for 2 seconds
                    println!("{buffer}");
                    break;
                }
            }

            // Print any remaining buffer
            if !buffer.is_empty() {
                println!("{buffer}");
            }
        }
    }

    Ok(())
}

fn send_command(port: &mut Box<dyn SerialPort>, cmd: &[u8]) -> Result<()> {
    port.write_all(cmd)?;
    Ok(())
}

fn read_response(port: &mut Box<dyn SerialPort>) -> Result<String> {
    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];

    loop {
        if port.read_exact(&mut byte).is_err() {
            continue;
        }

        if byte[0] == b'\r' || byte[0] == b'\n' {
            if !buffer.is_empty() {
                break;
            }
            continue;
        }

        buffer.push(byte[0]);
    }

    Ok(String::from_utf8_lossy(&buffer).to_string())
}

fn read_all_response(port: &mut Box<dyn SerialPort>, verbose: bool) -> Result<(String, Vec<u8>)> {
    use std::time::Instant;

    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];
    let mut lines = Vec::new();
    let mut data_bytes = Vec::new();
    let start = Instant::now();
    let max_wait = Duration::from_secs(60);
    let mut last_activity = Instant::now();
    let mut in_hex_dump = false;
    let mut started = false;

    loop {
        if start.elapsed() > max_wait {
            if verbose {
                eprintln!("Timeout after 60 seconds");
            }
            break;
        }

        match port.read_exact(&mut byte) {
            Ok(_) => {
                last_activity = Instant::now();

                if byte[0] == b'\r' || byte[0] == b'\n' {
                    if buffer.is_empty() {
                        continue;
                    }

                    let line = String::from_utf8_lossy(&buffer).to_string();

                    if !started
                        && (line.contains("Requesting variable")
                            || line.contains("Listening for incoming transfer"))
                    {
                        started = true;
                    }

                    if started {
                        if verbose {
                            eprintln!("← {line}");
                        }

                        if line.contains("Received") && line.contains("bytes:") {
                            in_hex_dump = true;
                        } else if in_hex_dump {
                            // Stop hex dump parsing if we hit a protocol message or status line
                            if line.starts_with("R")
                                || line.starts_with("S")
                                || line.contains("Transfer complete")
                                || line.contains("OK")
                                || line.contains("ERROR")
                            {
                                in_hex_dump = false;
                            } else {
                                // Parse all hex tokens from this line (skip non-hex content)
                                for token in line.split_whitespace() {
                                    if let Ok(val) = u8::from_str_radix(token, 16) {
                                        data_bytes.push(val);
                                    }
                                }
                            }
                        }

                        lines.push(line.clone());

                        if line.contains("OK") || line.contains("ERROR") {
                            break;
                        }
                    } else if verbose {
                        eprintln!("← {line} [skipped]");
                    }

                    buffer.clear();
                    continue;
                }

                buffer.push(byte[0]);
            }
            Err(_) => {
                if started && last_activity.elapsed() > Duration::from_secs(65) && !lines.is_empty()
                {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }

    Ok((lines.join("\n"), data_bytes))
}
