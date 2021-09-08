use clap::{App, load_yaml};

fn main() {
    let yaml = load_yaml!("args.yml");
    let args = App::from(yaml).get_matches();
    //let a = libpci::access::get("linux-sysfs");

    //for v in a {
    //    println!("{:?}", v);
    //}
    match args.subcommand() {
        Some(("list", matches)) => {
        },
        Some(("set", matches)) => { todo!() },
        _ => {},
    }
}
