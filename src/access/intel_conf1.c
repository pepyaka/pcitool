#include <stdint.h>
#include <sys/io.h>
#define CONFIG_ADDRESS 0xCF8
#define CONFIG_DATA 0xCFC

uint16_t pci_config_read_word (uint8_t bus, uint8_t slot, uint8_t func, uint8_t offset) {
    uint32_t address;
    uint32_t lbus  = (uint32_t)bus;
    uint32_t lslot = (uint32_t)slot;
    uint32_t lfunc = (uint32_t)func;
    uint16_t tmp = 0;
 
    /* create configuration address as per Figure 1 */
    address = (uint32_t)((lbus << 16) | (lslot << 11) |
              (lfunc << 8) | (offset & 0xfc) | ((uint32_t)0x80000000));
 
    iopl(3);
    /* write out the address */
    outl(CONFIG_ADDRESS, address);
    /* read in the data */
    /* (offset & 2) * 8) = 0 will choose the first word of the 32 bits register */
    tmp = (uint16_t)((inl(CONFIG_DATA) >> ((offset & 2) * 8)) & 0xffff);
    iopl(0);
    return (tmp);
}
//if (ioperm(MY_BASEPORT, 3, 1)) {
//         /* handle error */
//}
//
//if (ioperm(MY_BASEPORT, 3, 0)) {
//         /* handle error */
//}
//
//un pci_config_read(bus: u8, device: u8, func: u8, offset: u8) -> u32 {
//    let bus = bus as u32;
//    let device = device as u32;
//    let func = func as u32;
//    let offset = offset as u32;
//    // construct address param
//    let address = ((bus << 16) | (device << 11) | (func << 8) | (offset & 0xfc) | 0x80000000) as u32;
//
//if (ioperm(MY_BASEPORT, 3, 1)) {
//         /* handle error */
//}
//
//if (ioperm(MY_BASEPORT, 3, 0)) {
//         /* handle error */
//}
//    // write address
//    write_to_port(0xCF8, address);
//
//    // read data
//    read_from_port(0xCFC)
//}
//
//#[allow(dead_code)]
//unsafe fn pci_config_write(bus: u8, device: u8, func: u8, offset: u8, value: u32) {
//    let bus = bus as u32;
//    let device = device as u32;
//    let func = func as u32;
//    let offset = offset as u32;
//    // construct address param
//    let address = ((bus << 16) | (device << 11) | (func << 8) | (offset & 0xfc) | 0x80000000) as u32;
//
//    // write address
//    write_to_port(0xCF8, address);
//
//    // write data
//    write_to_port(0xCFC, value)
//}
