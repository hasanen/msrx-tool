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
use output::OutputFormat;

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
    /// Output format separator when using combined output format
    output_format_separator: Option<char>,
}
#[derive(Parser, Debug)]
enum CliCommand {
    #[clap(name = "read")]
    /// Read all tracks
    Read,
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
            let result = msrx_device.read_tracks(&args.data_format.unwrap()).unwrap();
            println!(
                "{}",
                output::format(
                    &result,
                    &args.output_format.unwrap(),
                    &args.output_format_separator,
                )
            );
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

    println!("Resetting device");
    // msrx_device.
    // match msrx_device.device_handle.reset() {
    //     Ok(_) => {}
    //     Err(e) => {
    //         println!("Error: {}", e);
    //         process::exit(1);
    //     }
    // }

    // let _ = msrx_device.release_interface();
    // let _ = msrx_device.attach_kernel_driver();
}
