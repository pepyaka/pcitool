use pcics::Header;
use pcitool::{access::Access, device::Device, names::Names};

fn main() {
    let names = Names::init().unwrap();
    let vds = names.vendor_device_subsystem();
    let cc = names.class_code();
    // Hope the device at address "00:00.0" always exists
    let address = "00:00.0".parse().unwrap();
    // Use first available access method
    let access = Access::init().unwrap();
    let device = access.device(address).unwrap();
    let Device {
        address,
        header:
            Header {
                vendor_id,
                device_id,
                class_code,
                command,
                status,
                ..
            },
        ..
    } = &device;
    println!(
        "{:#} {} [{:02x}] (Sub: {:02x}, ProgIf: {:02x}): {} [{:04x}] ({} [{:04x}])",
        address,
        cc.lookup(class_code.base, None, None).unwrap_or_default(),
        class_code.base,
        class_code.sub,
        class_code.interface,
        vds.lookup(*vendor_id, None, None).unwrap_or_default(),
        vendor_id,
        vds.lookup(*vendor_id, *device_id, None).unwrap_or_default(),
        device_id
    );
    println!("{:#?}", command);
    println!("{:#?}", status);

    println!("Capabilities:");
    if let Some(caps) = device.capabilities() {
        for cap in caps.flatten() {
            println!("{:x?}", cap.kind);
        }
    } else {
        println!("<access denied>");
    }
}
