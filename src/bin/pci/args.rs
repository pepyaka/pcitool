#[cfg(feature = "clap")]
use clap::Parser;
use clap::{builder::TypedValueParser, ErrorKind};

use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, about, version)]
pub struct Args {
    #[clap(short = 'G')]
    pub debug: bool,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// List all PCI devices
    #[clap(name = "list", alias = "ls")]
    List(List),
    /// Configure PCI devices
    #[clap(name = "set")]
    Set(Set),
}

#[derive(Parser, Debug)]
pub struct List {
    // /// Produce machine-readable output (single -m for an obsolete format)
    // #[clap(short = 'm', parse(from_occurrences))]
    // pub machine: usize,
    // /// Show bus tree
    // #[clap(short = 't')]
    // pub tree: bool,
    /// Show hex-dump of the standard part of the config space
    #[clap(short = 'x', parse(from_occurrences))]
    pub hex: usize,

    /// Instead of accessing real hardware, read the list of devices and values of their configuration registers from the given file
    #[clap(short = 'F', value_name = "file")]
    pub file: Option<PathBuf>,

    /// Be verbose (-vv or -vvv for higher verbosity)
    #[clap(short = 'v', parse(from_occurrences))]
    pub verbose: usize,
    /// Show kernel drivers handling each device
    #[clap(short = 'k')]
    pub kernel: bool,
    // /// Bus-centric view (addresses and IRQ's as seen by the bus)
    // #[cfg_attr(feature = "clap", clap(short = 'b'))]
    // pub bus_centric: bool,
    /// Always show domain numbers
    #[clap(short = 'D')]
    pub always_domain_number: bool,
    // /// Display bridge path in addition to bus and device number
    // #[cfg_attr(feature = "clap", clap(short = 'P', parse(from_occurrences)))]
    // pub path_through: usize,
    /// Show numeric ID's
    #[clap(short = 'n', parse(from_occurrences))]
    pub as_numbers: usize,

    /// The library supports a variety of methods to access the PCI hardware.
    /// By default, it uses the first access method available, but you can use this
    /// option to override this decision.
    #[clap(short = 'A', value_enum, value_name = "method")]
    pub method: Option<PreferredMethod>,

    /// The behavior of the library is controlled by several named parameters.
    /// This option allows one to set the value of any of the parameters.
    #[clap(short = 'O', value_name = "param>=<value", value_parser = ParameterValueParser)]
    pub(crate) parameter_value: Option<ParameterValue>,
    
    // This option actuallly does not work
    // #[clap(short = 'p', value_name = "file")]
    // pub(crate) modules_alias: Option<PathBuf>,
    
    /// Use <file> as the PCI ID list instead of /usr/share/hwdata/pci.ids.
    #[clap(short = 'i', value_name = "file")]
    pub(crate) pci_ids_path: Option<PathBuf>,
    
    /// Show only devices in the specified domain (in case your machine has several
    /// host bridges, they can either share  a  common  bus  number space  or  each  of
    /// them can address a PCI domain of its own; domains are numbered from 0 to ffff),
    /// bus (0 to ff), device (0 to 1f) and function (0 to 7).  Each component of the
    /// device address can be omitted or set to "*", both meaning "any value". All
    /// numbers are  hexa‐ decimal.  E.g., "0:" means all devices on bus 0, "0" means
    /// all functions of device 0 on any bus, "0.3" selects third function of device 0
    /// on all buses and ".4" shows only the fourth function of each device.
    #[clap(short = 's', value_name = "[[[[<domain>]:]<bus>]:][<device>][.[<func>]]")]
    pub(crate) address: Option<PathBuf>,
    

}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum PreferredMethod {
    LinuxSysfs,
    #[clap(name = "linux-proc")]
    LinuxProcfs,
    IntelConf1,
    IntelConf2,
    #[cfg(target_os = "freebsd")]
    FbsdDevice,
    #[cfg(target_os = "netbsd")]
    NbsdDevice,
    #[cfg(target_os = "openbsd")]
    ObsdDevice,
    #[cfg(target_os = "macos")]
    Darwin,
    Dump,
}

#[derive(Debug, Clone)]
// #[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub(crate) enum ParameterValue {
    // #[clap(name = "dump.name")]
    DumpName(PathBuf),
    ProcPath(PathBuf),
    SysfsPath(PathBuf),
    NetCacheName(PathBuf),
    NetDomain(String),
}

#[derive(Debug, Clone)]
pub(crate) struct ParameterValueParser;

impl TypedValueParser for ParameterValueParser {
    type Value = ParameterValue;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let mut cmd = cmd.clone();
        let (param, value) = value
            .to_str()
            .and_then(|s| s.split_once('='))
            .ok_or_else(|| cmd.error(ErrorKind::InvalidValue, "format is <key> = <value>"))?;
        match param {
            "dump.name" => Ok(ParameterValue::DumpName(PathBuf::from(value))),
            "proc.path" => Ok(ParameterValue::ProcPath(PathBuf::from(value))),
            "sysfs.path" => Ok(ParameterValue::SysfsPath(PathBuf::from(value))),
            "net.cache_name" => Ok(ParameterValue::NetCacheName(PathBuf::from(value))),
            "net.domain" => Ok(ParameterValue::NetDomain(value.into())),
            _ => Err(cmd.error(
                ErrorKind::InvalidValue,
                "available values: dump.name, proc.path, sysfs.path, net.cache_name, net.domain",
            )),
        }
    }
}

#[derive(Parser, Debug)]
pub struct Set;
