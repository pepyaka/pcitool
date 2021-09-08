//use std::process::{Command, Stdio};
//use pretty_assertions::assert_eq;
//
//mod common;
//use common::*;
//
//const INXI_PATH: &str = concat![env!("CARGO_MANIFEST_DIR"), "/tests/data/inxi/inxi"];
//const INXIRS_PATH: &str = env!("CARGO_BIN_EXE_inxirs");
//
//#[test]
//#[ignore]
//fn system() {
//    sudo("-c0 -a -S", &[]);
//}
//
//#[test]
//#[ignore]
//fn machine() {
//    let args_str_list = &[
//        "-c0 -M", // Same with any args: -x, -xx, -xxx, -a
//        "-c0 -M --dmidecode",
//        "-c0 -x -M --dmidecode",
//        "-c0 -xx -M --dmidecode",
//        "-c0 -xxx -M --dmidecode",
//        "-c0 -a -M --dmidecode",
//    ];
//    for args_str in args_str_list {
//        sudo(args_str, &[]);
//    }
//}
//
//#[test]
//#[ignore]
//fn memory() {
//    let args_str_list = &[
//        "-c0 -m",
//        "-c0 -x -m",
//        "-c0 -xx -m",
//        "-c0 -xxx -m",
//        "-c0 -a -m --dmidecode",
//        "-c0 -x -m --dmidecode",
//        "-c0 -xx -m --dmidecode",
//        "-c0 -xxx -m --dmidecode",
//        "-c0 -a -m --dmidecode",
//    ];
//    for args_str in args_str_list {
//        sudo(args_str, MEMORY_SUBS);
//    }
//}
//
//
//#[test]
//#[ignore]
//fn battery() {
//    let args_str_list = &[
//        "-c0 -B",
//        "-c0 -x -B",
//        "-c0 -xx -B",
//        "-c0 -xxx -B",
//        "-c0 -a -B --dmidecode",
//        "-c0 -x -B --dmidecode",
//        "-c0 -xx -B --dmidecode",
//        "-c0 -xxx -B --dmidecode",
//        "-c0 -a -B --dmidecode",
//    ];
//    for args_str in args_str_list {
//        sudo(args_str, BATTERY_SUBS);
//    }
//}
//
//#[test]
//#[ignore]
//fn cpu() {
//    let args_str_list = &[
//        "-c0 -C",
//        "-c0 -x -C",
//        "-c0 -xx -C",
//        "-c0 -xxx -C",
//        "-c0 -a -C --dmidecode",
//        "-c0 -x -C --dmidecode",
//        "-c0 -xx -C --dmidecode",
//        "-c0 -xxx -C --dmidecode",
//        "-c0 -a -C --dmidecode",
//    ];
//    for args_str in args_str_list {
//        sudo(args_str, CPU_SUBS);
//    }
//}
//
//#[test]
//#[ignore]
//fn sensors() {
//    sudo("-c0 -a -s", SENSORS_SUBS);
//}
//
//#[test]
//#[ignore]
//fn info() {
//    sudo("-c0 -a -I", INFO_SUBS);
//}
//
////#[test]
////fn () {
////    sudo("-c0 -a -S", EMPTY_SUBS);
////}
//
//fn sudo(args_str: &str, subs: &[(&str,&str)]) {
//    let args: Vec<&str> = args_str.split_whitespace().collect();
//    let inxi = Command::new("sudo")
//        .arg("-E")
//        .arg("--")
//        .arg("perl")
//        .arg("--")
//        .arg(INXI_PATH)
//        .args(&args)
//        .stdin(Stdio::inherit())
//        .output()
//        .expect("failed to execute inxi");
//    let inxi_out = String::from_utf8_lossy(&inxi.stdout);
//    let inxi_err = String::from_utf8_lossy(&inxi.stderr);
//
//    let inxirs = Command::new("sudo")
//        .arg("-E")
//        .arg("--")
//        .arg(&INXIRS_PATH)
//        .args(&args)
//        .stdin(Stdio::inherit())
//        .output()
//        .expect("failed to execute inxirs");
//    let inxirs_out = String::from_utf8_lossy(&inxirs.stdout);
//    let inxirs_err = String::from_utf8_lossy(&inxirs.stderr);
//    
//    assert_eq!(inxi_err, inxirs_err);
//
//    // printing raw stdouts without any substitutions
//    println!("Args: {:?}", &args_str);
//    println!("inxi:\n{}\ninxirs:\n{}\n", &inxi_out, &inxirs_out);
//    
//    let inxi_out = substitute_values(&inxi_out, subs);
//    let inxirs_out = substitute_values(&inxirs_out, subs);
//    
//    assert_text(&inxi_out, &inxirs_out);
//}

//#[test]
//fn intel_conf1() {
//    let  = Command::new("sudo")
//        .arg("-E")
//        .arg("--")
//        .arg(&INXIRS_PATH)
//        .args(&args)
//        .stdin(Stdio::inherit())
//        .output()
//        .expect("failed to execute inxirs");
//    let inxirs_out = String::from_utf8_lossy(&inxirs.stdout);
//    let inxirs_err = String::from_utf8_lossy(&inxirs.stderr);
//    
//    assert_eq!(inxi_err, inxirs_err);
//
//    // printing raw stdouts without any substitutions
//    println!("Args: {:?}", &args_str);
//    println!("inxi:\n{}\ninxirs:\n{}\n", &inxi_out, &inxirs_out);
//    
//    let inxi_out = substitute_values(&inxi_out, subs);
//    let inxirs_out = substitute_values(&inxirs_out, subs);
//    
//    assert_text(&inxi_out, &inxirs_out);
//}
