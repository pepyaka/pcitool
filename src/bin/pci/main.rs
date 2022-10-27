use std::path::PathBuf;

use clap::Parser;

use pcitool::{
    access::{self, dump::Dump, linux_procfs::LinuxProcfs, linux_sysfs::LinuxSysfs, Access, Void},
    names::Names,
    view::lspci,
};

mod args;
use args::{Args, Command, List, ParameterValue, PreferredMethod};

fn main() {
    let args = Args::parse();
    match args.command {
        Command::List(args) => list(args),
        _ => todo!(),
    }
}

fn list(args: List) {
    let List {
        method,
        file,
        verbose,
        as_numbers,
        kernel,
        always_domain_number,
        parameter_value,
        pci_ids_path,
        ..
    } = args;

    let linux_sysfs = if let Some(ParameterValue::SysfsPath(ref path)) = parameter_value {
        LinuxSysfs::new(path)
    } else {
        LinuxSysfs::default()
    };
    // if let Some(path) = modules_alias {
    //     linux_sysfs.modules_alias_path(path);
    // }

    let result: access::Result<Access> = match (method, file) {
        (_, Some(path)) => Dump::init(path).map(Into::into),
        (Some(PreferredMethod::Dump), None) => Dump::init("/dev/stdin").map(Into::into),
        (Some(PreferredMethod::LinuxSysfs), _) => linux_sysfs.access(),
        (Some(PreferredMethod::LinuxProcfs), _) => {
            let path = if let Some(ParameterValue::ProcPath(path)) = parameter_value {
                path
            } else {
                PathBuf::from(LinuxProcfs::PATH)
            };
            LinuxProcfs::init(path).map(Into::into)
        }
        _ => linux_sysfs
            .access()
            .or_else(|_| LinuxProcfs::init(LinuxProcfs::PATH).map(Into::into))
            .or_else(|_| Void::init().map(Into::into)),
    };

    // Print errors to stderr
    let access = result.unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1)
    });

    // Split successfully parse devices and errors
    let (devices, errors): (Vec<_>, Vec<_>) = access.iter().partition(Result::is_ok);
    let mut devices: Vec<_> = devices.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

    devices.sort();
    // Show domain (slot) if any device domain != 0000
    let always_domain_number =
        always_domain_number || devices.iter().any(|d| d.address.domain != 0);
    let names = if let Some(pci_ids_path) = pci_ids_path {
        Names::init_pciids(pci_ids_path).unwrap_or_default()
    } else {
        Names::init().unwrap_or_default()
    };
    let vds = &names.vendor_device_subsystem();
    let cc = &names.class_code();
    let args = &lspci::basic::ViewArgs {
        verbose,
        kernel,
        always_domain_number,
        as_numbers,
        vds,
        cc,
        access: &access,
    };
    for data in devices {
        print!("{}", lspci::basic::View { data, args });
    }
    for error in &errors {
        print!("{}", error);
    }
}
