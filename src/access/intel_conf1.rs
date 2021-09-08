

extern "C" {
    pub fn pci_config_read_word (bus: u8, slot: u8, func: u8, offset: u8) -> u16;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extern_c() {
        //unsafe {
        //    let r = pci_config_read_word(0, 0, 0, 0);
        //    
        //    dbg!(r);
        //}
    }
}


















//struct PciMethods {
//    name: *mut u8,
//    help: *mut u8,
//}
//
////enum pci_access_type {
////  /* Known access methods, remember to update init.c as well */
////  PCI_ACCESS_AUTO,			/* Autodetection */
////  PCI_ACCESS_SYS_BUS_PCI,		/* Linux /sys/bus/pci */
////  PCI_ACCESS_PROC_BUS_PCI,		/* Linux /proc/bus/pci */
////  PCI_ACCESS_I386_TYPE1,		/* i386 ports, type 1 */
////  PCI_ACCESS_I386_TYPE2,		/* i386 ports, type 2 */
////  PCI_ACCESS_FBSD_DEVICE,		/* FreeBSD /dev/pci */
////  PCI_ACCESS_AIX_DEVICE,		/* /dev/pci0, /dev/bus0, etc. */
////  PCI_ACCESS_NBSD_LIBPCI,		/* NetBSD libpci */
////  PCI_ACCESS_OBSD_DEVICE,		/* OpenBSD /dev/pci */
////  PCI_ACCESS_DUMP,			/* Dump file */
////  PCI_ACCESS_DARWIN,			/* Darwin */
////  PCI_ACCESS_SYLIXOS_DEVICE,		/* SylixOS pci */
////  PCI_ACCESS_HURD,			/* GNU/Hurd */
////  PCI_ACCESS_MAX
////};
//
//#[repr(C)]
//struct PciAccess {
//  /* Options you can change: */
//  method: c_uint,			/* Access method */
//  writeable: c_int,			/* Open in read/write mode */
//  buscentric: c_int,			/* Bus-centric view of the world */
//
//  id_file_name: *mut c_char,			/* Name of ID list file (use pci_set_name_list_path()) */
//  free_id_name: c_int,			/* Set if id_file_name is malloced */
//  numeric_ids: c_int,			/* Enforce PCI_LOOKUP_NUMERIC (>1 => PCI_LOOKUP_MIXED) */
//
//  id_lookup_mode: c_uint,		/* pci_lookup_mode flags which are set automatically */
//					/* Default: PCI_LOOKUP_CACHE */
//
//  debugging: c_int,			/* Turn on debugging messages */
//
//  /* Functions you can override: */
//  //error: *mut c_void,	/* Write error message and quit */
//  //warning: *mut c_void,	/* Write a warning message */
//  //debug: *mut c_void,	/* Write a debugging message */
//
//  devices: *mut PciDev,		/* Devices found on this bus */
//
//  /* Fields used internally: */
//  methods: *mut PciMethods,
//  params: *mut PciParam,
//  id_hash: *mut IdEntry,		/* names.c */
//  current_id_bucket: *mut IdBucket,
//  id_load_failed: c_int,
//  id_cache_status: c_int,		/* 0=not read, 1=read, 2=dirty */
//  //id_udev: *mut Udev,			/* names-hwdb.c */
//  id_udev: *mut c_void,			/* names-hwdb.c */
//  //id_udev_hwdb: *mut UdevHwdb,
//  id_udev_hwdb: *mut c_void,
//  fd: c_int,				/* proc/sys: fd for config space */
//  fd_rw: c_int,				/* proc/sys: fd opened read-write */
//  fd_pos: c_int,				/* proc/sys: current position */
//  fd_vpd: c_int,				/* sys: fd for VPD */
//  cached_dev: *mut PciDev,		/* proc/sys: device the fds are for */
//}
//
//#[repr(C)]
//struct Udev;
//
//#[repr(C)]
//struct UdevHwdb;
//
//#[repr(C)]
//struct PciParam {
//  next: *mut PciParam,		/* Please use pci_walk_params() for traversing the list */
//  param: *mut c_char,				/* Name of the parameter */
//  value: *mut c_char,				/* Value of the parameter */
//  value_malloced: c_int,			/* used internally */
//  help: *mut c_char,				/* Explanation of the parameter */
//}
//
//#[repr(C)]
//struct IdEntry {
//  next: *mut IdEntry,
//  id12: u32, id34: u32,
//  cat: u8,
//  src: u8,
//  name: [c_char; 1],
//}
//
//#[repr(C)]
//struct PciDev {
//  next: *mut PciDev,			/* Next device in the chain */
//  domain_u16: u16,			/* 16-bit version of the PCI domain for backward compatibility */
//					/* 0xffff if the real domain doesn't fit in 16 bits */
//  bus: u8, dev: u8, func: u8,			/* Bus inside domain, device and function */
//
//  /* These fields are set by pci_fill_info() */
//  known_fields: c_uint,		/* Set of info fields already known (see pci_fill_info()) */
//  vendor_id: u16, device_id: u16,		/* Identity of the device */
//  device_class: u16,			/* PCI device class */
//  irq: c_int,				/* IRQ number */
//  base_addr: [PciAddr; 6],		/* Base addresses including flags in lower bits */
//  size: [PciAddr; 6],		/* Base addresses including flags in lower bits */
//  rom_base_addr: PciAddr,		/* Expansion ROM base address */
//  rom_size: PciAddr,			/* Expansion ROM size */
//  first_cap: *mut PciCap,		/* List of capabilities */
//  phy_slot: *mut u8,			/* Physical slot */
//  module_alias: *mut u8,			/* Linux kernel module alias */
//  label: *mut u8,				/* Device name as exported by BIOS */
//  numa_node: c_int,			/* NUMA node */
//  flags: [PciAddr; 6],			/* PCI_IORESOURCE_* flags for regions */
//  rom_flags: PciAddr,			/* PCI_IORESOURCE_* flags for expansion ROM */
//  domain: c_int,				/* PCI domain (host bridge) */
//
//  /* Fields used internally */
//  access: *mut PciAccess,
//  methods: *mut PciMethods,
//  cache: *mut u8,				/* Cached config registers */
//  cache_len: c_int,
//  hdrtype: c_int,				/* Cached low 7 bits of header type, -1 if unknown */
//  aux: *mut c_void,				/* Auxiliary data for use by the back-end */
//  properties: *mut PciProperty,	/* A linked list of extra properties */
//  last_cap: *mut PciCap,		/* Last capability in the list */
//}
//
//#[repr(C)]
//struct PciProperty {
//  next: *mut PciProperty,
//  key: u32,
//  value: [c_char; 1],
//}
//
//#[repr(C)]
//struct IdBucket {
//  next: *mut IdBucket,
//  full: c_uint,
//}
//
//#[repr(C)]
//struct PciCap {
//  next: *mut PciCap,
//  id: u16,				/* PCI_CAP_ID_xxx */
//  r#type: u16,				/* PCI_CAP_xxx */
//  addr: c_uint,			/* Position in the config space */
//}
//
//
////struct pci_filter {
////  int domain, bus, slot, func;			/* -1 = ANY */
////  int vendor, device, device_class;
////  int rfu[3];
////};
//
////enum pci_lookup_mode {
////  PCI_LOOKUP_VENDOR = 1,		/* Vendor name (args: vendorID) */
////  PCI_LOOKUP_DEVICE = 2,		/* Device name (args: vendorID, deviceID) */
////  PCI_LOOKUP_CLASS = 4,			/* Device class (args: classID) */
////  PCI_LOOKUP_SUBSYSTEM = 8,
////  PCI_LOOKUP_PROGIF = 16,		/* Programming interface (args: classID, prog_if) */
////  PCI_LOOKUP_NUMERIC = 0x10000,		/* Want only formatted numbers; default if access->numeric_ids is set */
////  PCI_LOOKUP_NO_NUMBERS = 0x20000,	/* Return NULL if not found in the database; default is to print numerically */
////  PCI_LOOKUP_MIXED = 0x40000,		/* Include both numbers and names */
////  PCI_LOOKUP_NETWORK = 0x80000,		/* Try to resolve unknown ID's by DNS */
////  PCI_LOOKUP_SKIP_LOCAL = 0x100000,	/* Do not consult local database */
////  PCI_LOOKUP_CACHE = 0x200000,		/* Consult the local cache before using DNS */
////  PCI_LOOKUP_REFRESH_CACHE = 0x400000,	/* Forget all previously cached entries, but still allow updating the cache */
////  PCI_LOOKUP_NO_HWDB = 0x800000,	/* Do not ask udev's hwdb */
////};
//
//// Depends on platform
//type PciAddr = u64;
//
