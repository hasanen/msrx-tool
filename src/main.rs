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
use clap::CommandFactory;
use msrx_tool_error::MsrxToolError;
use msrx_tool_error::MsrxToolError::CardNotSwiped;
use output::OutputFormat;
use std::time::Duration;
use tracks_data::TracksData;

/// Simple tool for reading and writing data to magstripe devices
///
/// ## Errors
///
/// If there is an error during exection, the program will exit with a non-zero exit code. Error message will be printed to STDERR.
///
/// Codes:
///  1 - Generic error  
///  2 - Card not swiped/Timeout. Card was not swiped when expected
///
/// ## Allowed charaacters
///   Track 1: !"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ\^_
///   Track 2 & 3: 0123456789:;<=>?

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    arg_required_else_help = true,
    verbatim_doc_comment
)]
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
    #[clap(long, default_value = "20")]
    /// Timeout in seconds for writing tracks
    write_timeout: Option<u64>,
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
#[derive(Copy, Clone, Debug)]
enum ExitCode {
    Success = 0,
    CardNotSwiped = 2,
    GenericError = 1,
}
impl ExitCode {
    fn as_i32(&self) -> i32 {
        *self as i32
    }
}

fn main() {
    let args = Args::parse();
    CliCommand::command()
        .after_help("Some more text")
        .get_matches();

    let mut msrx_device = match MsrxDevice::init_msrx6() {
        Ok(device) => device,
        Err(e) => {
            println!("Error: {}", e);
            process::exit(1);
        }
    };

    match msrx_device.setup_device() {
        Ok(_) => {}
        Err(e) => {
            handle_error(&e);
        }
    }

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
                Err(e) => handle_error(&e),
            }
        }
        Some(CliCommand::Write { track_data }) => {
            let timeout = Duration::from_secs(args.write_timeout.unwrap());
            let separator = &args.format_separator.unwrap();
            match TracksData::from_str(track_data, &separator) {
                Ok(data) => match msrx_device.write_tracks(&data, &timeout) {
                    Ok(_) => println!("Write operation successful"),
                    Err(e) => handle_error(&e),
                },
                Err(e) => handle_error(&e),
            }
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

    process::exit(ExitCode::Success.as_i32());
}

fn handle_error(error: &MsrxToolError) {
    let exit_code = match error {
        CardNotSwiped => ExitCode::CardNotSwiped,
        _ => ExitCode::GenericError,
    };

    eprintln!("Error: {}", &error);
    process::exit(exit_code.as_i32());
}
