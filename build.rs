fn main() {
    println!("cargo:rerun-if-changed=src/access/intel_conf1.c");
    cc::Build::new()
        .file("src/access/intel_conf1.c")
        .compile("intel_conf1");
}
