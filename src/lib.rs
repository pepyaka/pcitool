/*!
```rust
# use pcitool::access::Access;
// Use first available access method
let access = Access::init().unwrap();
// First device
let zero = access.device("00:00.0".parse().unwrap());
// Fail if no device on "0000:00:00.0" address
assert!(zero.is_ok());
// Iterator through devices
let devices = access.iter();
// More than one device are in the system
assert!(devices.count() > 1);

```
*/

pub mod access;
pub mod device;
pub mod misc;
pub mod names;
pub mod view;
