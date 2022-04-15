#[cfg(feature = "clap")]
use clap::Parser;

use std::path::PathBuf;

use pcitool::{
    view::lspci::BasicView,
    access::PreferredMethod,
};

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



#[derive(Parser, Debug)]
pub struct Set;
