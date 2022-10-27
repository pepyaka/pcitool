//use std::{
//    fs,
//    io,
//    path::{Path, PathBuf},
//};

// use regex::Regex;
use pretty_assertions::assert_str_eq;

use std::ffi::OsStr;
use std::process::Command;
use std::process::Stdio;

const PCI_BIN_PATH: &str = env!("CARGO_BIN_EXE_pci");
pub const LSPCI_MUSL_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/bin/lspci-musl");

//pub static CPU_SUBS: &[(&str, &str)] = &[
//    (r"(Speed:) \d+ (MHz)", r"$1 0000 $2"),
//    (r"(Core speeds \(MHz\):)(\s+(\d+): (\d+))+", r"$1 $3: 0000"),
//    // inxi show incorrect L3 cache size
//    (r"L3 cache: \d+(\.\d{1,2})?", r"L3 cache: 0000"),
//];

#[allow(dead_code)]
pub static INFO_SUBS: &[(&str, &str)] = &[
    (r"Processes: \d+", r"Procs: 666"),
    (r"Uptime: \d+d \d+h \d+m", r"Uptime: 1d 2h 49m"),
    (r"used: \d+\.\d+? ([MGT]iB) \(\d+\.\d%\)", r"used: 0.00 $1 (00.0%)"),
    // inxi takes shell from parent process
    (r"running in: \S+", r"running in: cargo"),
    (r"Shell: \S+( v: \S+)?", r"Shell: integration-e45"),
    (r"running-in: \S+", r"running-in: cargo"),
    (r"inxi(rs)?: \S+", r"inxi: 0.0.0"),
];

//pub static SENSORS_SUBS: &[(&str, &str)] = &[
//    (r"charge: \d+\.\d Wh", "charge: 0.0 Wh"),
//    (r"cpu: \d+\.\d C mobo:", "cpu: 45.1 C mobo:"),
//    (r"temp: \d+\.\d C ", "temp: 45.1 C "),
//    (r"cpu: \d+ ", "cpu: 8888 "),
//];

#[allow(dead_code)]
pub static MEMORY_SUBS: &[(&str, &str)] = &[
    (r"used: \d+(\.\d{1,2})? [KMGTP]iB \(\d{2}.\d%\)", "used: 8.00 GiB (50.0%)"),
];

//pub static BATTERY_SUBS: &[(&str, &str)] = &[
//    (r"charge: (\d\d.\d) Wh \(\d\d.\d%\)", "charge: $1 Wh (98.9%)"),
//    (r"status: (Dis|Not )charging", "status: Discharging"),
//    (r"Lithium Ion", "Lithium-ion"),
//];

// pub(crate) fn substitute_values(input: &str, subs: &[(&str, &str)]) -> String {
//     let mut data = String::from(input);
//     for (pattern, val) in subs {
//         let re = Regex::new(pattern).unwrap();
//         data = re.replace_all(&data, *val).to_string();
//     }
//     data
// }

// pub(crate) fn assert_text(a: &str, b: &str) {
//     let a_lines = a.lines();
//     //let a_num = &a_lines.count();
//     let b_lines = b.lines();
//     //let b_num = &b_lines.count();
//     for (n, (a, b)) in a_lines.zip(b_lines).enumerate() {
//         assert_eq!(a, b, "Line #{} not equal", n);
//     }
//     assert_eq!(a.lines().count(), b.lines().count(), "Lines number different");
// }

//pub fn get_vfs_paths() -> Vec<PathBuf> {
//    let base_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/vfs");
//    fs::read_dir(base_path)
//        .and_then(|entries| {
//            entries.filter_map(|entry| {
//                entry.map(|entry| {
//                    let path = entry.path();
//                    if path.is_dir() {
//                        Some(path)
//                    } else {
//                        None
//                    }
//                })
//                .transpose()
//            })
//            .collect::<io::Result<Vec<_>>>()
//        })
//        .unwrap_or_default()
//}

pub(crate) fn compare_exe_outputs(lspci_path: impl AsRef<OsStr>, args_str: &str, test_stderr: bool) {
    let args: Vec<&str> = args_str.split_whitespace().collect();
    let lspci = Command::new(lspci_path)
        .args(&args)
        .stdin(Stdio::inherit())
        .output()
        .expect("failed to execute lspci");
    let lspci_out = String::from_utf8_lossy(&lspci.stdout);
    let lspci_err = String::from_utf8_lossy(&lspci.stderr);

    let pci_ls = Command::new(PCI_BIN_PATH)
        .arg("list")
        .args(&args)
        .stdin(Stdio::inherit())
        .output()
        .unwrap_or_else(|_| {
            panic!(
            "failed to execute `{} list`, probably you should build with --features=\"clap kmod\"",
            PCI_BIN_PATH
        )
        });
    let pci_ls_out = String::from_utf8_lossy(&pci_ls.stdout);
    let pci_ls_err = String::from_utf8_lossy(&pci_ls.stderr);

    dbg!(&lspci_err, &pci_ls_err);

    if test_stderr {
        assert_str_eq!(lspci_err, pci_ls_err, "STDERR");
    }

    assert_str_eq!(lspci_out, pci_ls_out, "STDOUT");
}