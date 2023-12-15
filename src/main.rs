use rusb::{Context, DeviceHandle, UsbContext};
use std::process;

mod char_bits_conversion;
mod command;
mod config;
mod msrx;
mod msrx_tool_error;
mod raw_device_data;
mod raw_tracks_data;
mod read;
mod reverse_string;
mod to_hex;
mod track_data;
mod track_status;
use crate::to_hex::ToHex;
use clap::Parser;
use command::Command;
use config::DeviceConfig;
use msrx::{MsrxDevice, MSRX};
use msrx_tool_error::MsrxToolError;
use raw_device_data::RawDeviceData;
use raw_tracks_data::RawTracksData;
use track_data::TrackType;

/// Simple tool for reading and writing data to magstripe devices
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None,arg_required_else_help = true)]
struct Args {
    /// Command to use: read
    #[clap(subcommand)]
    command: Option<CliCommand>,
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
    let _ = msrx_device.detach_kernel_driver();
    let _ = msrx_device.claim_interface();

    match msrx_device.device_handle.reset() {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            process::exit(1);
        }
    }

    let _ = msrx_device.set_bit_control_parity();
    let _ = msrx_device.set_hico_loco_mode();
    let _ = msrx_device.set_bit_per_inches();
    let _ = msrx_device.set_leading_zeros();

    match &args.command {
        Some(CliCommand::Read) => {
            let _result = read::read_all_tracks().unwrap();
        }

        Some(CliCommand::Firmware) => {
            let firmware = msrx_device.device_handle.get_firmware_version().unwrap();
            println!("{}", firmware);
        }

        Some(CliCommand::Model) => {
            let model = msrx_device.get_model().unwrap();
            println!("{}", model);
        }
        None => todo!(),
    }
    let _ = msrx_device.release_interface();
    let _ = msrx_device.attach_kernel_driver();
}
