# pcitool

This package provides [tool](src/bin) and library for portable access to PCI bus configuration registres.

Following systems are supported:

- [x] Linux
   - [x] [The procfs filesystem](https://en.wikipedia.org/wiki/Procfs)
   - [x] [The sysfs filesystem](https://en.wikipedia.org/wiki/Sysfs)
   - [ ] via i386 ports
- [ ] FreeBSD		(via /dev/pci)
- [ ] NetBSD		(via libpci)
- [ ] OpenBSD		(via /dev/pci)
- [ ] GNU/kFreeBSD	(via /dev/pci)
- [ ] Solaris/i386	(direct port access)
- [ ] Aix		(via /dev/pci and odmget)
- [ ] GNU Hurd	(direct port access)
- [ ] Windows		(via cfgmgr32 or direct port access, see README.Windows for caveats)
- [ ] CYGWIN		(direct port access)
- [ ] BeOS		(via syscalls)
- [ ] Haiku		(via /dev/misc/poke)
- [ ] Darwin		(via IOKit)
- [ ] DOS/DJGPP	(via i386 ports)
- [ ] SylixOS		(via /proc/pci)

Pcitool inspired by [pciutils](https://github.com/pciutils/pciutils) and tries to be compatible in every way

### Usage

#### List all PCI devices
- Brief: `pcitool ls`
- Verbose view: `pcitool ls -v`
- Show PCI vendor and device codes as both numbers and names: `pcitool ls -nn`

#### Configure PCI device
TODO