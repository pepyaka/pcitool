use std::process::Command;
use std::process::Stdio;
use pretty_assertions::assert_eq;


const PCI_BIN_PATH: &str = env!("CARGO_BIN_EXE_pci");
const F_ARG: &str = concat!("-F ", env!("CARGO_MANIFEST_DIR"), "/tests/data/lspci.dump");

mod common;
use common::*;

macro_rules! read_from_dump {
    ($($fname:ident: $args:expr,)*) => {
        $(
            // #[ignore] 
            #[test]
            fn $fname() {
                ls(&format!("{} {}", F_ARG, $args), &[]);
            }
        )*
    }
}

read_from_dump! {
    read_from_dump:       "",
    read_from_dump_n:     "-n",
    read_from_dump_nn:    "-nn",
    read_from_dump_v:     "-v",
    read_from_dump_vv:    "-vv",
    read_from_dump_vvv:   "-vvv",
    read_from_dump_nv:    "-nv",
    read_from_dump_nvv:   "-nvv",
    read_from_dump_nvvv:  "-nvvv",
    read_from_dump_nnv:   "-nnv",
    read_from_dump_nnvv:  "-nnvv",
    read_from_dump_nnvvv: "-nnvvv",
}

fn ls(args_str: &str, subs: &[(&str, &str)]) {
    let args: Vec<&str> = args_str.split_whitespace().collect();
    let lspci = Command::new("lspci")
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
        .expect("failed to execute `pci list`");
    let pci_ls_out = String::from_utf8_lossy(&pci_ls.stdout);
    let pci_ls_err = String::from_utf8_lossy(&pci_ls.stderr);
    
    assert_eq!(lspci_err, pci_ls_err);

    // printing raw stdouts without any substitutions
    println!("Args: {:?}", &args_str);
    println!("lspci:\n{}\npci ls:\n{}\n", &lspci_out, &pci_ls_out);
    
    let lspci_sub = substitute_values(&lspci_out, subs);
    let pci_ls_sub = substitute_values(&pci_ls_out, subs);
    
    assert_text(&lspci_sub, &pci_ls_sub);
}
