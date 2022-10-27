use std::{
    fs,
    io::{self, BufRead},
    path::Path,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ModulesAlias {
    data: Vec<(String, String)>,
}

impl ModulesAlias {
    pub fn init(path: impl AsRef<Path>) -> io::Result<ModulesAlias> {
        let file = fs::File::open(path.as_ref())?;
        let buf_reader = io::BufReader::new(file);
        let data = buf_reader
            .lines()
            .filter_map(|entry| {
                let line = entry.ok()?;
                let mut fields = line.split_ascii_whitespace().skip(1);
                let pattern = fields.next().filter(|p| p.starts_with("pci:"))?.to_string();
                let value = fields.next()?.to_string();
                Some((pattern, value))
            })
            .collect();
        Ok(Self { data })
    }
    pub fn lookup<'a>(&'a self, modalias: &'a str) -> impl Iterator<Item = String> + 'a {
        self.data.iter().filter_map(|(pattern, value)| {
            let pattern = glob::Pattern::new(pattern).ok()?;
            pattern.matches(modalias).then(|| value.clone())
        })
    }
}
