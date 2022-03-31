## Interrupt PIN
All values fixed to be less than (('A' as u32) + (*v as u32) - 1) < 128
## Capabilities
lspci wrongly shows capabilityis pointed to Header body, so all pointers fixed to be > 0x40
