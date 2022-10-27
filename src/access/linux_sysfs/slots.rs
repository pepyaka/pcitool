use std::{collections::HashMap, fs, io, path::Path};

use crate::device::Address;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Slots {
    data: HashMap<Address, String>,
}

impl Slots {
    pub fn init(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let entries = fs::read_dir(&path)?;
        let data = entries
            .filter_map(|entry| {
                let key = entry.ok()?.file_name().to_str()?.to_string();
                let path = path.join(&key).join("address");
                let mut address = fs::read_to_string(&path).ok()?.trim().to_string();
                address.push_str(".0");
                address.parse().ok().map(|val| (val, key))
            })
            .collect::<HashMap<_, _>>();
        Ok(Self { data })
    }
    pub fn find(&self, addr: impl Into<Address>) -> Option<String> {
        self.data.get(&addr.into()).cloned()
    }
}
