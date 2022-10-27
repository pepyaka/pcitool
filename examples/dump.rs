use pcitool::access::{dump::Dump, AccessMethod};

const DUMP: &str = r#"
00:1f.5 Serial bus controller [0c80]: Intel Corporation Cannon Lake PCH SPI Controller [8086:a324] (rev 10)
        Subsystem: Dell Device [1028:088e]
        Control: I/O- Mem+ BusMaster+ SpecCycle- MemWINV- VGASnoop- ParErr- Stepping- SERR- FastB2B- DisINTx+
        Status: Cap- 66MHz- UDF- FastB2B- ParErr- DEVSEL=fast >TAbort- <TAbort- <MAbort- >SERR- <PERR- INTx-
        Latency: 0
        Region 0: Memory at fe010000 (32-bit, non-prefetchable) [size=4K]
00: 86 80 24 a3 06 04 00 00 10 00 80 0c 00 00 00 00
10: 00 00 01 fe 00 00 00 00 00 00 00 00 00 00 00 00
20: 00 00 00 00 00 00 00 00 00 00 00 00 28 10 8e 08
30: 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
"#;

fn main() {
    let access = Dump::new(DUMP.to_string());
    for device in access.iter().flatten() {
        println!("{:#x?}", device);
    }
}
