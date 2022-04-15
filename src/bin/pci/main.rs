use clap::Parser;

use pcitool::{
    access::{
        AccessMethod,
        PreferredMethod,
        dump::Dump
    },
    pciids::{PciIds, VendorDeviceSubsystem, ClassCode},
    view::{DisplayMultiViewBasic, lspci::LspciView},
};

mod args;
use args::{Args, List, Command};

fn main() {
    let args = Args::parse();
    let (vendor_device_subsystem, class_code) =
        PciIds::new(include_str!("/usr/share/hwdata/pci.ids").lines())
            .collect::<(VendorDeviceSubsystem, ClassCode)>();
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
                    let view = LspciView { basic_view, vendor_device_subsystem, class_code };
                    for device in devices {
                        print!("{}", device.display(&view));
                    }
                },
                (Some(PreferredMethod::Dump), None) => todo!("Read dump from stdin"),
                m => todo!("Access method {:?} not implemented", m),
            }
        },
        _ => todo!(),
    }
}
