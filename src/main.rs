use std::process;

mod char_bits_conversion;
mod command;
mod config;
mod device_data;
mod msrx;
mod msrx_tool_error;
mod reverse_string;
mod to_hex;
mod track_data;
mod track_status;
mod tracks_data;
use clap::Parser;
use msrx::MsrxDevice;
mod data_format;
use data_format::DataFormat;
mod iso_data;
mod original_device_data;
mod output;
mod processing_format;
use msrx_tool_error::MsrxToolError::DeviceError;
use output::OutputFormat;
use std::time::Duration;
use tracks_data::TracksData;

/// Simple tool for reading and writing data to magstripe devices
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None,arg_required_else_help = true)]
struct Args {
    /// Command to use: read
    #[clap(subcommand)]
    command: Option<CliCommand>,
    #[clap(short, long, default_value = "iso")]
    /// Data format to use: iso, raw
    data_format: Option<DataFormat>,
    #[clap(short, long, default_value = "combined")]
    /// Output format: json or stdout
    output_format: Option<OutputFormat>,
    #[clap(long, default_value = "_")]
    /// Input/output format separator when using combined output format
    format_separator: Option<char>,
    #[clap(long, default_value = "20")]
    /// Timeout in seconds for reading tracks
    read_timeout: Option<u64>,
}
#[derive(Parser, Debug)]
enum CliCommand {
    #[clap(name = "read")]
    /// Read all tracks
    Read,
    #[clap(name = "write")]
    /// Write content to tracks. Use
    Write { track_data: String },
    #[clap(name = "fw")]
    /// Print firmware of the device
    Firmware,
    #[clap(name = "model")]
    /// Print model of the device
    Model,
}

fn main() {
    let args = Args::parse();

    let mut msrx_device = match MsrxDevice::init_msrx6() {
        Ok(device) => device,
        Err(e) => {
            println!("Error: {}", e);
            process::exit(1);
        }
    };

    msrx_device.setup_device().unwrap();

    match &args.command {
        Some(CliCommand::Read) => {
            let timeout = Duration::from_secs(args.read_timeout.unwrap());

            match msrx_device.read_tracks(&args.data_format.unwrap(), &timeout) {
                Ok(result) => {
                    println!(
                        "{}",
                        output::format(
                            &result,
                            &args.output_format.unwrap(),
                            &args.format_separator,
                        )
                    );
                }
                Err(e) => match e {
                    DeviceError(rusb::Error::Timeout) => {
                        // msrx_device.disable_read_mode();
                    }
                    _ => {
                        dbg!(&e);
                        println!("Error2: {}", e);
                    }
                },
            }
        }
        Some(CliCommand::Write { track_data }) => {
            let separator = &args.format_separator.unwrap();
            let data = TracksData::from_str(track_data, &separator).unwrap();
            msrx_device.write_tracks(&data).ok();
        }

        Some(CliCommand::Firmware) => {
            let firmware = msrx_device.get_firmware_version().unwrap();
            println!("{}", firmware);
        }

        Some(CliCommand::Model) => {
            let model = msrx_device.get_model().unwrap();
            println!("{}", model);
        }
        None => todo!(),
    }
}
