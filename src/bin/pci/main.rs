use std::path::{Path, PathBuf};

use clap::Clap;

use libpci::{
    access::{
        AccessMethod,
        PreferredMethod,
        dump::Dump
    },
    pciids::{PciIds, VendorDeviceSubsystem},
    view::{
        DisplayMultiView,
        lspci::LspciDevice,
    }
};

mod args;
use args::{Args, List, Command};

fn main() {
    let args = Args::parse();
    let (vds, _) = PciIds::new(include_str!("/usr/share/hwdata/pci.ids").lines())
        .collect::<(VendorDeviceSubsystem, _)>();
    match args.command {
        Command::List(List { mut method, file, mut basic_view, .. }) => {
            if let (None, Some(_)) = (&method, &file) {
                method = Some(PreferredMethod::Dump);
            }
            match (method, file) {
                (None | Some(PreferredMethod::Dump), Some(path))  => {
                    let data = std::fs::read_to_string(path)
                        .unwrap();
                    let dump = Dump::new(data);
                    let mut devices = dump.iter().collect::<Vec<_>>();
                    devices.sort();
                    // Show domain (slot) if any != 0000
                    if devices.iter().find(|d| d.address.domain != 0).is_some() {
                        basic_view.always_domain_number = true;
                    }
                    for device in devices {
                        let device = LspciDevice { device: &device, vds: &vds };
                        print!("{}", device.display(&basic_view));
                    }
                },
                (Some(PreferredMethod::Dump), None) => todo!("Read dump from stdin"),
                m => todo!("Access method {:?} not implemented", m),
            }
        },
        _ => todo!(),
    }
}
