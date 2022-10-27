#![cfg(target_os = "linux")]

mod common;
use common::{compare_exe_outputs, LSPCI_MUSL_PATH};

const PCI_IDS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/pci.ids");

macro_rules! user_dump_multiple_args {
    ($($fname:ident: $x:expr, $args:expr,)*) => {
        $(
            // #[ignore]
            #[test]
            fn $fname() {
                let dump = format!("{}/tests/data/machine/362f18e/out.{}.txt", env!("CARGO_MANIFEST_DIR"), $x);
                compare_exe_outputs(LSPCI_MUSL_PATH, &format!("-i {} -F {} {}", PCI_IDS_PATH, dump, $args), true);
            }
        )*
    }
}

user_dump_multiple_args! {
    args_x:       "x", "",
    args_x_n:     "x", "-n",
    args_x_nn:    "x", "-nn",
    args_x_v:     "x", "-v",
    args_x_vv:    "x", "-vv",
    args_x_vvv:   "x", "-vvv",
    args_x_nv:    "x", "-nv",
    args_x_nvv:   "x", "-nvv",
    args_x_nvvv:  "x", "-nvvv",
    args_x_nnv:   "x", "-nnv",
    args_x_nnvv:  "x", "-nnvv",
    args_x_nnvvv: "x", "-nnvvv",
    args_xxx:       "xxx", "",
    args_xxx_n:     "xxx", "-n",
    args_xxx_nn:    "xxx", "-nn",
    args_xxx_v:     "xxx", "-v",
    args_xxx_vv:    "xxx", "-vv",
    args_xxx_vvv:   "xxx", "-vvv",
    args_xxx_nv:    "xxx", "-nv",
    args_xxx_nvv:   "xxx", "-nvv",
    args_xxx_nvvv:  "xxx", "-nvvv",
    args_xxx_nnv:   "xxx", "-nnv",
    args_xxx_nnvv:  "xxx", "-nnvv",
    args_xxx_nnvvv: "xxx", "-nnvvv",
    args_xxxx:       "xxxx", "",
    args_xxxx_n:     "xxxx", "-n",
    args_xxxx_nn:    "xxxx", "-nn",
    args_xxxx_v:     "xxxx", "-v",
    // This test cover most of issues
    args_xxxx_vv:    "xxxx", "-vv",
    args_xxxx_vvv:   "xxxx", "-vvv",
    args_xxxx_nv:    "xxxx", "-nv",
    args_xxxx_nvv:   "xxxx", "-nvv",
    args_xxxx_nvvv:  "xxxx", "-nvvv",
    args_xxxx_nnv:   "xxxx", "-nnv",
    args_xxxx_nnvv:  "xxxx", "-nnvv",
    args_xxxx_nnvvv: "xxxx", "-nnvvv",
}

macro_rules! machines {
    ($($fname:ident: $machine:expr,)*) => {
        $(
            // #[ignore]
            #[test]
            fn $fname() {
                compare_exe_outputs(LSPCI_MUSL_PATH, &format!(
                    "-F {}/tests/data/machine/{}/out.xxxx.txt -nnvvv -i {}",
                    env!("CARGO_MANIFEST_DIR"),
                    $machine,
                    PCI_IDS_PATH,
                ), true);
            }
        )*
    }
}

machines! {
    machine_ec8a5fc: "ec8a5fc",
    machine_02daadc: "02daadc",
    machine_23c7a39: "23c7a39",
}

#[cfg(test)]
mod fuzzing {
    use super::*;
    use seq_macro::seq;
    use std::{io::Write, iter};
    use tempfile::NamedTempFile;

    const RANDOM_DATA: &[u8; 4096 * 64] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/fuzzing/random"
    ));

    const TEST_COUNT: usize = if cfg!(feature = "expensive_tests") {
        64
    } else {
        1
    };

    const CAP_EA_FIXES: [(usize, u8, u8); 12 * 3] = [
        (0x44, 0b111, 0x02),
        (0x48, 0b10, 0x00),
        (0x4c, 0b10, 0x00),
        (0x50, 0b111, 0x03),
        (0x54, 0b10, 0x00),
        (0x58, 0b10, 0x10),
        (0x60, 0b111, 0x03),
        (0x64, 0b10, 0x10),
        (0x68, 0b10, 0x00),
        (0x70, 0b111, 0x04),
        (0x74, 0b10, 0x10),
        (0x78, 0b10, 0x10),
        (0x84, 0b111, 0x02),
        (0x88, 0b10, 0x00),
        (0x8c, 0b10, 0x00),
        (0x90, 0b111, 0x03),
        (0x94, 0b10, 0x00),
        (0x98, 0b10, 0x10),
        (0xa0, 0b111, 0x03),
        (0xa4, 0b10, 0x10),
        (0xa8, 0b10, 0x00),
        (0xb0, 0b111, 0x04),
        (0xb4, 0b10, 0x10),
        (0xb8, 0b10, 0x10),
        (0xc4, 0b111, 0x02),
        (0xc8, 0b10, 0x00),
        (0xcc, 0b10, 0x00),
        (0xd0, 0b111, 0x03),
        (0xd4, 0b10, 0x10),
        (0xd8, 0b10, 0x00),
        (0xe0, 0b111, 0x03),
        (0xe4, 0b10, 0x00),
        (0xe8, 0b10, 0x10),
        (0xf0, 0b111, 0x03),
        (0xf4, 0b10, 0x10),
        (0xf8, 0b10, 0x00),
    ];

    #[derive(Clone, Copy)]
    enum Param {
        Header { htype: u8 },
        Caps { id: u8, htype: u8 },
        Ecaps { id: u16, htype: u8, aux: usize },
    }

    #[test]
    fn header_type_00() {
        run_test(Param::Header { htype: 0x00 });
    }

    #[test]
    fn header_type_00_multifunction() {
        run_test(Param::Header { htype: 0x08 });
    }

    #[test]
    fn header_type_01() {
        run_test(Param::Header { htype: 0x01 });
    }

    #[test]
    fn header_type_01_multifunction() {
        run_test(Param::Header { htype: 0x01 | 0x80 });
    }

    #[test]
    fn header_type_02() {
        run_test(Param::Header { htype: 0x02 });
    }

    #[test]
    fn header_type_02_multifunction() {
        run_test(Param::Header { htype: 0x02 | 0x80 });
    }

    // use paste::paste;
    seq!(N in 0x00..=0x15 {
        #[test]
        fn capability_~N() {
            run_test(Param::Caps { id: N, htype: 0 });
        }
    });

    #[test]
    fn capability_10_bridge() {
        run_test(Param::Caps { id: 10, htype: 1 });
    }

    #[test]
    fn capability_14_bridge() {
        run_test(Param::Caps { id: 14, htype: 1 });
    }

    seq!(N in 0x00..=0x34 {
        #[test]
        fn extended_capability_~N() {
            run_test(Param::Ecaps { id: N, htype: 0, aux: 0 });
        }
    });

    // Compute Express Link
    #[test]
    fn extended_capability_23_cxl() {
        run_test(Param::Ecaps {
            id: 0x23,
            htype: 0,
            aux: 0x1e980000,
        });
    }

    fn run_test(param: Param) {
        let dump: String = RANDOM_DATA
            .chunks_exact(4096)
            .take(TEST_COUNT)
            .chain(iter::once([u8::MIN; 4096].as_slice()))
            .chain(iter::once([u8::MAX; 4096].as_slice()))
            .enumerate()
            .map(|(dn, cs)| {
                let conf_space = &mut [0u8; 4096];
                conf_space.copy_from_slice(cs);
                let len = add_fixtures(conf_space, param);

                let (bus, dev, fun) = ((dn & 0xff00) >> 8, (dn & 0b11111000) >> 3, dn & 0b111);
                let body = conf_space[..len]
                    .chunks_exact(16)
                    .enumerate()
                    .map(|(ln, hbytes)| {
                        let hbytes: String = hbytes.iter().map(|b| format!(" {:02x}", b)).collect();
                        format!("{:x}0:{}\n", ln, hbytes)
                    });
                Some(format!("{:02x}:{:02x}.{:x} _\n", bus, dev, fun))
                    .into_iter()
                    .chain(body)
                    .collect::<String>()
            })
            .collect();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{}", dump).unwrap();
        let args = format!(
            "-nnvvv -F {} -i {}",
            file.path().to_string_lossy(),
            PCI_IDS_PATH
        );
        dbg!(&args);

        // Uncoment to save temp file
        file.keep().unwrap();

        compare_exe_outputs(LSPCI_MUSL_PATH, &args, true);
    }

    fn add_fixtures(slice: &mut [u8], param: Param) -> usize {
        let mut fill_common_caps = |id, htype| {
            slice[..64].fill(0);
            slice[0x06] = 0x10; // Has capabilities
            slice[0x0e] = htype; // Header type
            slice[0x34] = 0x40; // Cap ptr
            slice[0x40] = id; // Cap ID
            slice[0x41] = 0; // Next cap ID
        };
        match param {
            Param::Header { htype } => {
                slice[0x0e] = htype;
                slice[0x34] = slice[0x34].max(0x40); // Cap pointer should be >= 0x40
                slice[0x3d] &= 0xf; // Limit interrupt pin to 16
                64
            }
            Param::Caps { id: 0x14, htype } => {
                fill_common_caps(0x14, htype);
                slice[0x42] &= 0xf; // Entries number
                for (mut offset, mask, val) in CAP_EA_FIXES {
                    if htype == 1 {
                        offset += 4;
                    }
                    slice[offset] = slice[offset] & !mask | val;
                }
                256
            }
            Param::Caps { id, htype } => {
                fill_common_caps(id, htype);
                256
            }
            Param::Ecaps { id, htype, aux } => {
                let _ = htype;
                slice[..256].fill(0);
                slice[0x06] = 0x10; // Has capabilities
                slice[0x34] = 0x40; // Cap ptr
                slice[0x40] = 0x10; // PCI Express
                let [lo, hi] = id.to_le_bytes();
                (slice[0x100], slice[0x101]) = (lo, hi); // Ecap ID
                (slice[0x102], slice[0x103]) = (0, 0); // Next ecap ID
                if aux == 0x1e980000 {
                    slice[0x104..0x10A].copy_from_slice(&[0x98, 0x1E, 0x81, 0x03, 0x00, 0x00]);
                }
                4096
            }
        }
    }
}
