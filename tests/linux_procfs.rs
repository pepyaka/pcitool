#![cfg(target_os = "linux")]

mod common;
use common::{compare_exe_outputs, LSPCI_MUSL_PATH};

#[test]
fn vfs_machine_caf6526() {
    let args = "-vvvnn";
    let method = "linux-proc";
    let opts = concat!(
        "proc.path=",
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/machine/caf6526/vfs/proc/bus/pci"
    );
    let pci_ids = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/pci.ids");
    compare_exe_outputs(
        LSPCI_MUSL_PATH,
        &format!("{args} -A {method} -O {opts} -i {pci_ids}"),
        true,
    );
}
