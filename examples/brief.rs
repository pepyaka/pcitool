use pcics::Header;
use pcitool::{access::Access, device::Device, names::Names};

fn main() {
    let names = Names::init().unwrap();
    let vds = names.vendor_device_subsystem();
    let cc = names.class_code();
    // Use first available access method
    let access = Access::init().unwrap();
    // Iterate over device addresses
    for device in access.iter().flatten() {
        let Device {
            address,
            header:
                Header {
                    vendor_id,
                    device_id,
                    class_code,
                    ..
                },
            ..
        } = device;
        println!(
            "{:#} {} [{:02x}] (Sub: {:02x}, ProgIf: {:02x}): {} [{:04x}] ({} [{:04x}])",
            address,
            cc.lookup(class_code.base, None, None).unwrap_or_default(),
            class_code.base,
            class_code.sub,
            class_code.interface,
            vds.lookup(vendor_id, None, None).unwrap_or_default(),
            vendor_id,
            vds.lookup(vendor_id, device_id, None).unwrap_or_default(),
            device_id
        );
    }
}
