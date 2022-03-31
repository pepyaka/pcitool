use std::process::Command;
use std::process::Stdio;
use pretty_assertions::assert_eq;


const PCI_BIN_PATH: &str = env!("CARGO_BIN_EXE_pci");

// mod common;

macro_rules! user_dump_multiple_args {
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
                ), true);
            }
        )*
    }
}

user_dump_multiple_args! {
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

macro_rules! user_dump_nnvvv_multiple_machines {
    ($($fname:ident: $machine:expr,)*) => {
        $(
            // #[ignore] 
            #[test]
            fn $fname() {
                ls(&format!(
                    "-F {}/tests/data/machine/{}/out.xxxx.txt -nnvvv",
                    env!("CARGO_MANIFEST_DIR"),
                    $machine,
                ), true);
            }
        )*
    }
}

user_dump_nnvvv_multiple_machines! {
    user_xxxx_dump_ec8a5fc_nnvvv: "ec8a5fc",
    user_xxxx_dump_02daadc_nnvvv: "02daadc",
    user_xxxx_dump_23c7a39_nnvvv: "23c7a39",
}

macro_rules! user_dump_fuzzing_nnvvv {
    ($($fname:ident: $fuzz:expr,)*) => {
        $(
            // #[ignore] 
            #[test]
            fn $fname() {
                ls(&format!(
                    "-F {}/tests/data/fuzzing/{} -nnvvv",
                    env!("CARGO_MANIFEST_DIR"),
                    $fuzz,
                ), false);
            }
        )*
    }
}

user_dump_fuzzing_nnvvv! {
    user_dump_fuzzing_header_type0_nnvvv: "header/Type00",
    user_dump_fuzzing_header_type0_multifunction_nnvvv: "header/Type00.multi",
    user_dump_fuzzing_header_type1_nnvvv: "header/Type01",
    user_dump_fuzzing_header_type1_multifunction_nnvvv: "header/Type01.multi",
    user_dump_fuzzing_header_type2_nnvvv: "header/Type02",

    user_dump_fuzzing_capability_00_nnvvv: "capability/00",
    user_dump_fuzzing_capability_01_nnvvv: "capability/01",
    user_dump_fuzzing_capability_02_nnvvv: "capability/02",
    user_dump_fuzzing_capability_03_nnvvv: "capability/03",
    user_dump_fuzzing_capability_04_nnvvv: "capability/04",
    user_dump_fuzzing_capability_05_nnvvv: "capability/05",
    user_dump_fuzzing_capability_06_nnvvv: "capability/06",
    user_dump_fuzzing_capability_07_nnvvv: "capability/07",
    user_dump_fuzzing_capability_08_nnvvv: "capability/08",
    user_dump_fuzzing_capability_09_nnvvv: "capability/09",
    user_dump_fuzzing_capability_0a_nnvvv: "capability/0a",
    user_dump_fuzzing_capability_0b_nnvvv: "capability/0b",
    user_dump_fuzzing_capability_0c_nnvvv: "capability/0c",
    user_dump_fuzzing_capability_0d_nnvvv: "capability/0d",
    user_dump_fuzzing_capability_0e_nnvvv: "capability/0e",
    user_dump_fuzzing_capability_0f_nnvvv: "capability/0f",
    user_dump_fuzzing_capability_10_nnvvv: "capability/10",
    user_dump_fuzzing_capability_11_nnvvv: "capability/11",
    user_dump_fuzzing_capability_12_nnvvv: "capability/12",
    user_dump_fuzzing_capability_13_nnvvv: "capability/13",
    user_dump_fuzzing_capability_14_nnvvv: "capability/14",
    user_dump_fuzzing_capability_15_nnvvv: "capability/15",
    
    user_dump_fuzzing_extended_capability_00_nnvvv: "capability/00",
    user_dump_fuzzing_extended_capability_01_nnvvv: "capability/01",
    user_dump_fuzzing_extended_capability_02_nnvvv: "capability/02",
    user_dump_fuzzing_extended_capability_03_nnvvv: "capability/03",
    user_dump_fuzzing_extended_capability_04_nnvvv: "capability/04",
    user_dump_fuzzing_extended_capability_05_nnvvv: "capability/05",
    user_dump_fuzzing_extended_capability_06_nnvvv: "capability/06",
    user_dump_fuzzing_extended_capability_07_nnvvv: "capability/07",
    user_dump_fuzzing_extended_capability_08_nnvvv: "capability/08",
    user_dump_fuzzing_extended_capability_09_nnvvv: "capability/09",
    user_dump_fuzzing_extended_capability_0a_nnvvv: "capability/0a",
    user_dump_fuzzing_extended_capability_0b_nnvvv: "capability/0b",
    user_dump_fuzzing_extended_capability_0c_nnvvv: "capability/0c",
    user_dump_fuzzing_extended_capability_0d_nnvvv: "capability/0d",
    user_dump_fuzzing_extended_capability_0e_nnvvv: "capability/0e",
    user_dump_fuzzing_extended_capability_0f_nnvvv: "capability/0f",
    user_dump_fuzzing_extended_capability_10_nnvvv: "capability/10",
    user_dump_fuzzing_extended_capability_11_nnvvv: "capability/11",
    user_dump_fuzzing_extended_capability_12_nnvvv: "capability/12",
    user_dump_fuzzing_extended_capability_13_nnvvv: "capability/13",
    user_dump_fuzzing_extended_capability_14_nnvvv: "capability/14",
    user_dump_fuzzing_extended_capability_15_nnvvv: "capability/15",
    user_dump_fuzzing_extended_capability_16_nnvvv: "capability/16",
    user_dump_fuzzing_extended_capability_17_nnvvv: "capability/17",
    user_dump_fuzzing_extended_capability_18_nnvvv: "capability/18",
    user_dump_fuzzing_extended_capability_19_nnvvv: "capability/19",
    user_dump_fuzzing_extended_capability_1a_nnvvv: "capability/1a",
    user_dump_fuzzing_extended_capability_1b_nnvvv: "capability/1b",
    user_dump_fuzzing_extended_capability_1c_nnvvv: "capability/1c",
    user_dump_fuzzing_extended_capability_1d_nnvvv: "capability/1d",
    user_dump_fuzzing_extended_capability_1e_nnvvv: "capability/1e",
    user_dump_fuzzing_extended_capability_1f_nnvvv: "capability/1f",
    user_dump_fuzzing_extended_capability_20_nnvvv: "capability/20",
    user_dump_fuzzing_extended_capability_21_nnvvv: "capability/21",
    user_dump_fuzzing_extended_capability_22_nnvvv: "capability/22",
    user_dump_fuzzing_extended_capability_23_nnvvv: "capability/23",
    user_dump_fuzzing_extended_capability_24_nnvvv: "capability/24",
    user_dump_fuzzing_extended_capability_25_nnvvv: "capability/25",
    user_dump_fuzzing_extended_capability_26_nnvvv: "capability/26",
    user_dump_fuzzing_extended_capability_27_nnvvv: "capability/27",
    user_dump_fuzzing_extended_capability_28_nnvvv: "capability/28",
    user_dump_fuzzing_extended_capability_29_nnvvv: "capability/29",
    user_dump_fuzzing_extended_capability_2a_nnvvv: "capability/2a",
    user_dump_fuzzing_extended_capability_2b_nnvvv: "capability/2b",
    user_dump_fuzzing_extended_capability_2c_nnvvv: "capability/2c",
}

fn ls(args_str: &str, test_stderr: bool) {
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
        .expect(&format!(
            "failed to execute `{} list`, probably you should build with --features=\"clap kmod\"",
            PCI_BIN_PATH
        ));
    let pci_ls_out = String::from_utf8_lossy(&pci_ls.stdout);
    let pci_ls_err = String::from_utf8_lossy(&pci_ls.stderr);
    
    let lspci_err_lines = lspci_err.lines().collect::<Vec<_>>();
    let pci_ls_err_lines = pci_ls_err.lines().collect::<Vec<_>>();
    
    if test_stderr {
        assert_eq!(lspci_err_lines, pci_ls_err_lines, "STDERR");
    }

    let lspci_out_lines = lspci_out.lines().collect::<Vec<_>>();
    let pci_ls_out_lines = pci_ls_out.lines().collect::<Vec<_>>();

    assert_eq!(lspci_out_lines, pci_ls_out_lines, "STDOUT");
}
