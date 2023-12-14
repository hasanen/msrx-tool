use rusb::{Context, DeviceHandle, UsbContext};
use std::process;

mod char_bits_conversion;
mod command;
mod config;
mod msrx;
mod msrx_tool_error;
mod raw_device_data;
mod raw_tracks_data;
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
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    device: String,

    /// Command to use: hours, integrations etc
    #[clap(subcommand)]
    command: Option<Command>,
}
#[derive(Parser, Debug)]
enum Command {
    #[clap(name = "read")]
    /// Read all tracks
    ReadCommand {
        #[clap(subcommand)]
        action: integrations::Action,
    },
}

fn main() {
    let args = Args::parse();

    dbg!(args);
    // let mut msrx_device = MsrxDevice::init_msrx6().unwrap();

    // dbg!(&msrx_device);

    // let _ = msrx_device.detach_kernel_driver();
    // let _ = msrx_device.claim_interface();

    // println!("Reset device");
    // msrx_device.device_handle.reset().unwrap();

    // println!("read firmware");
    // let firmware = msrx_device.device_handle.get_firmware_version().unwrap();
    // println!("Firmware: {}", firmware);

    // println!("Set BPC");
    // let _ = msrx_device.set_bit_control_parity();

    // println!("Set HiCo/LoCo mode");
    // let _ = msrx_device.set_hico_loco_mode();

    // println!("Set BPI");
    // let _ = msrx_device.set_bit_per_inches();

    // println!("Set leading zeros");
    // let _ = msrx_device.set_leading_zeros();

    // println!("Get model");
    // let model = msrx_device.get_model().unwrap();
    // println!("Firmware: {}", model);

    // println!("Read card");
    // let tracks = msrx_device.read_tracks().unwrap();
    // dbg!(&tracks.raw_device_data.raw_data.to_hex());
    // dbg!(&tracks.track3.raw);
    // println!(
    //     "Track 1: {:?}",
    //     tracks.track1.to_string_with_bpc(
    //         TrackType::Track1IsoAlphabet,
    //         msrx_device.config.track1.bpc as usize
    //     )
    // );
    // println!(
    //     "Track 2: {:?}",
    //     tracks.track2.to_string_with_bpc(
    //         TrackType::Track2_3IsoAlpahbet,
    //         msrx_device.config.track2.bpc as usize
    //     )
    // );
    // println!(
    //     "Track 3: {:?}",
    //     tracks.track3.to_string_with_bpc(
    //         TrackType::Track2_3IsoAlpahbet,
    //         msrx_device.config.track3.bpc as usize
    //     )
    // );
    // msrx_device.device_handle.reset().unwrap();

    // let _ = msrx_device.release_interface();
    // let _ = msrx_device.attach_kernel_driver();
}
