#[cfg(feature = "clap")]
use clap::{AppSettings, Clap};

use std::path::PathBuf;

use libpci::{
    view::lspci::BasicView,
    access::PreferredMethod,
};

#[derive(Debug, Clap)]
#[clap(author, about, version, setting = AppSettings::ColoredHelp)]
pub struct Args {
    #[clap(short = 'G')]
    pub debug: bool,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clap)]
pub enum Command {
    /// List all PCI devices
    #[clap(name = "list", alias = "ls")]
    List(List),
    /// Configure PCI devices
    #[clap(name = "set")]
    Set(Set),
}

#[derive(Debug, Clap)]
pub struct List {
    /// Produce machine-readable output (single -m for an obsolete format)
    #[clap(short = 'm', parse(from_occurrences))]
    pub machine: usize,
    /// Show bus tree
    #[clap(short = 't')]
    pub tree: bool,
    /// Show hex-dump of the standard part of the config space
    #[clap(short = 'x', parse(from_occurrences))]
    pub hex: usize,
    
    /// Instead of accessing real hardware, read the list of devices and values of their configuration registers from the given file
    #[clap(short = 'F')]
    pub file: Option<PathBuf>,

    #[clap(flatten)]
    pub basic_view: BasicView,
    
    #[clap(short = 'A', arg_enum)]
    pub method: Option<PreferredMethod>,
}



#[derive(Debug, Clap)]
pub struct Set;
