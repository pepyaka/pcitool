//use std::process::Command;
//use std::process::Stdio;
//use pretty_assertions::assert_eq;
//
//
//const PCI_BIN_PATH: &str = env!("CARGO_BIN_EXE_pci");
//
//mod common;
//use common::*;
//
//#[test]
//fn basic_display_modes() {
//    let args_str_list = &[
//        "",
//        //"-m",
//        //"-mm",
//        //"-t",
//    ];
//    for args_str in args_str_list {
//        ls(args_str, &[]);
//    }
//}
//
//
//fn ls(args_str: &str, subs: &[(&str, &str)]) {
//    let args: Vec<&str> = args_str.split_whitespace().collect();
//    let lspci = Command::new("lspci")
//        .args(&args)
//        .stdin(Stdio::inherit())
//        .output()
//        .expect("failed to execute inxi");
//    let lspci_out = String::from_utf8_lossy(&lspci.stdout);
//    let lspci_err = String::from_utf8_lossy(&lspci.stderr);
//
//    let pci_ls = Command::new(PCI_BIN_PATH)
//        .arg("list")
//        .args(&args)
//        .stdin(Stdio::inherit())
//        .output()
//        .expect("failed to execute inxirs");
//    let pci_ls_out = String::from_utf8_lossy(&pci_ls.stdout);
//    let pci_ls_err = String::from_utf8_lossy(&pci_ls.stderr);
//    
//    assert_eq!(lspci_err, pci_ls_err);
//
//    // printing raw stdouts without any substitutions
//    println!("Args: {:?}", &args_str);
//    println!("lspci:\n{}\npci ls:\n{}\n", &lspci_out, &pci_ls_out);
//    
//    let lspci_sub = substitute_values(&lspci_out, subs);
//    let pci_ls_sub = substitute_values(&pci_ls_out, subs);
//    
//    assert_text(&lspci_sub, &pci_ls_sub);
//}
