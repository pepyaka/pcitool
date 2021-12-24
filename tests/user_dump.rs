use std::process::Command;
use std::process::Stdio;
use pretty_assertions::assert_eq;


const PCI_BIN_PATH: &str = env!("CARGO_BIN_EXE_pci");

// mod common;

macro_rules! read_from_dump {
    ($($fname:ident: $x:expr, $args:expr,)*) => {
        $(
            // #[ignore] 
            #[test]
            fn $fname() {
                ls(&format!(
                    "-F {}/tests/data/machine/362f18e/out.{}.txt {}",
                    env!("CARGO_MANIFEST_DIR"),
                    $x,
                    $args,
                ));
            }
        )*
    }
}

read_from_dump! {
    user_x_dump:       "x", "",
    user_x_dump_n:     "x", "-n",
    user_x_dump_nn:    "x", "-nn",
    user_x_dump_v:     "x", "-v",
    user_x_dump_vv:    "x", "-vv",
    user_x_dump_vvv:   "x", "-vvv",
    user_x_dump_nv:    "x", "-nv",
    user_x_dump_nvv:   "x", "-nvv",
    user_x_dump_nvvv:  "x", "-nvvv",
    user_x_dump_nnv:   "x", "-nnv",
    user_x_dump_nnvv:  "x", "-nnvv",
    user_x_dump_nnvvv: "x", "-nnvvv",
    user_xxx_dump:       "xxx", "",
    user_xxx_dump_n:     "xxx", "-n",
    user_xxx_dump_nn:    "xxx", "-nn",
    user_xxx_dump_v:     "xxx", "-v",
    user_xxx_dump_vv:    "xxx", "-vv",
    user_xxx_dump_vvv:   "xxx", "-vvv",
    user_xxx_dump_nv:    "xxx", "-nv",
    user_xxx_dump_nvv:   "xxx", "-nvv",
    user_xxx_dump_nvvv:  "xxx", "-nvvv",
    user_xxx_dump_nnv:   "xxx", "-nnv",
    user_xxx_dump_nnvv:  "xxx", "-nnvv",
    user_xxx_dump_nnvvv: "xxx", "-nnvvv",
    user_xxxx_dump:       "xxxx", "",
    user_xxxx_dump_n:     "xxxx", "-n",
    user_xxxx_dump_nn:    "xxxx", "-nn",
    user_xxxx_dump_v:     "xxxx", "-v",
    // This test cover most of issues
    user_xxxx_dump_vv:    "xxxx", "-vv",
    user_xxxx_dump_vvv:   "xxxx", "-vvv",
    user_xxxx_dump_nv:    "xxxx", "-nv",
    user_xxxx_dump_nvv:   "xxxx", "-nvv",
    user_xxxx_dump_nvvv:  "xxxx", "-nvvv",
    user_xxxx_dump_nnv:   "xxxx", "-nnv",
    user_xxxx_dump_nnvv:  "xxxx", "-nnvv",
    user_xxxx_dump_nnvvv: "xxxx", "-nnvvv",
}

fn ls(args_str: &str) {
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

    let lspci_out_lines = lspci_out.lines().collect::<Vec<_>>();
    let pci_ls_out_lines = pci_ls_out.lines().collect::<Vec<_>>();

    assert_eq!(lspci_out_lines, pci_ls_out_lines);
}
