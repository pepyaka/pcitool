use std::{io, fs, path::{Path, PathBuf}};

use thiserror::Error;
use kmod;

use crate::device::Address;




#[derive(Debug)]
pub struct Kmod {
    pub driver: String,
    pub modules: Vec<String>,
}

#[derive(Error, Debug)]
pub enum KmodError {
    #[error("driver symlink reading problem")]
    DriverSymlink(#[source] io::Error),
    #[error("get last part of path problem")]
    FileName,
    #[error("can not convert file path to string")]
    ToStr,
    #[error("get driver dependiencies problem")]
    Modules(#[from] kmod::Error),
    #[error("modalias reading problem")]
    Modalias(#[source] io::Error),
}

impl Kmod {
    pub fn get(address: &Address) -> Result<Self, KmodError> {
        let path = Path::new("/sys/bus/pci/devices")
            .join(address.to_string());
        Kmod::get_with_dir_prefix(path)
    }
    pub fn get_with_dir_prefix(path: PathBuf) -> Result<Self, KmodError> {
        let driver = fs::read_link(path.join("driver"))
            .map_err(KmodError::DriverSymlink)?
            .file_name()
            .ok_or(KmodError::FileName)?
            .to_str()
            .ok_or(KmodError::ToStr)?
            .to_string();
        let modalias: std::ffi::OsString = fs::read_to_string(path.join("modalias"))
            .map_err(KmodError::Modalias)?
            .into();
        // create a new kmod context
        let ctx = kmod::Context::new()
            .map_err(KmodError::Modules)?;

        dbg!(&modalias);
        let modules = ctx.module_new_from_lookup(&modalias)
            .map_err(KmodError::Modules)?
            .map(|m| m.name().to_string())
            .collect::<Vec<_>>();
        Ok(Self { driver, modules })
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    #[ignore]
    fn kmod_get() {
        let test_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/device/8086:9dc8");
        let kmod = Kmod::get_with_dir_prefix(test_path).unwrap();
        assert_eq!("snd_hda_intel", kmod.driver, "driver");
        let modules_sample = vec!["snd_hda_intel", "snd_soc_skl", "snd_sof_pci_intel_cnl"];
        assert_eq!(modules_sample, kmod.modules, "modules");
    }
}
