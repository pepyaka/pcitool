#![cfg(all(target_os = "linux", not(feature = "pciutils_make_opt_libkmod")))]

mod common;
use common::{compare_exe_outputs, LSPCI_MUSL_PATH};

#[test]
fn vfs_machine_caf6526() {
    let args = "-vvvnn";
    let method = "linux-sysfs";
    let opts = concat!(
        "sysfs.path=",
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/machine/caf6526/vfs/sys/bus/pci"
    );
    let pci_ids = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/pci.ids");
    compare_exe_outputs(
        LSPCI_MUSL_PATH,
        &format!("{args} -A {method} -O {opts} -i {pci_ids}"),
        true,
    );
}

#[test]
fn vfs_machine_23c7a39() {
    let args = "-vvvnn";
    let method = "linux-sysfs";
    let opts = concat!(
        "sysfs.path=",
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/machine/23c7a39/vfs/sys/bus/pci"
    );
    let pci_ids = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/pci.ids");
    compare_exe_outputs(
        LSPCI_MUSL_PATH,
        &format!("{args} -A {method} -O {opts} -i {pci_ids}"),
        true,
    );
}
