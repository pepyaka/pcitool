use core::fmt;


use crate::device;

pub mod lspci;

mod header;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verbose(pub usize);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiView<T, V> {
    pub data: T,
    pub view: V,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum View {
    Basic,
    Extended,
    LspciBasic(lspci::BasicView),
    LspciMachineSimple,
}

pub trait DisplayMultiView<'a> {
    type Data;
    type View;
    fn display(&'a self, view: Self::View) -> MultiView<Self::Data, Self::View>;
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BoolView {
    CheckMark,
    PlusMinus,
    Str(&'static str),
}

impl<'a> fmt::Display for MultiView<bool, BoolView> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.data, self.view) {
            (false, BoolView::CheckMark) => write!(f, "✗"),
            (true, BoolView::CheckMark) => write!(f, "✓"),
            (false, BoolView::PlusMinus) => write!(f, "-"),
            (true, BoolView::PlusMinus) => write!(f, "+"),
            (true, BoolView::Str(s)) => write!(f, "{}", s),
            _ => Ok(()),
        }
    }
}
impl<'a> DisplayMultiView<'a> for bool {
    type Data = bool;
    type View = BoolView;
    fn display(&'a self, view: Self::View) -> MultiView<Self::Data, Self::View> {
        MultiView { data: *self, view, }
    }
}



use device::Address;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    Suppress,
    Always,
}

impl<'a> DisplayMultiView<'a> for Address {
    type Data = &'a Address;
    type View = Domain;
    fn display(&'a self, view: Self::View) -> MultiView<Self::Data, Self::View> {
        MultiView { data: self, view, }
    }
}
impl<'a> fmt::Display for MultiView<&'a Address, Domain> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Address { domain: dom, bus: b, device: dev, function: fun } = self.data;
        if self.view == Domain::Always || dom != &0 {
            write!(f, "{:04x}:{:02x}:{:02x}.{:x}", dom, b, dev, fun)
        } else {
            write!(f, "{:02x}:{:02x}.{:x}", b, dev, fun)
        }
    }
}


use device::capabilities::message_signaled_interrups::{
    MessageSignaledInterrups,
    MessageAddress
};

impl<'a> DisplayMultiView<'a> for MessageSignaledInterrups {
    type Data = &'a MessageSignaledInterrups;
    type View = Verbose;
    fn display(&'a self, view: Self::View) -> MultiView<Self::Data, Self::View> {
        MultiView { data: self, view, }
    }
}
impl<'a> fmt::Display for MultiView<&'a MessageSignaledInterrups, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (ctrl, addr, data, mask, pend) = (
            &self.data.message_control, 
            &self.data.message_address, 
            &self.data.message_data, 
            &self.data.mask_bits, 
            &self.data.pending_bits
        );
        let Verbose(verbose) = self.view;
        write!(f, "MSI: Enable{} Count={}/{} Maskable{} 64bit{}\n", 
            ctrl.enable.display(BoolView::PlusMinus),
            ctrl.multiple_message_enable as u8,
            ctrl.multiple_message_capable as u8,
            ctrl.per_vector_masking_capable.display(BoolView::PlusMinus),
            matches!(addr, MessageAddress::Qword(_)).display(BoolView::PlusMinus),
        )?;
        if verbose < 2 {
            return Ok(())
        }
        match addr {
            MessageAddress::Dword(v) => {
                write!(f, "\t\tAddress: {:08x}  Data: {:04x}\n", v, data)?;
            },
            MessageAddress::Qword(v) => {
                write!(f, "\t\tAddress: {:016x}  Data: {:04x}\n", v, data)?;
            },
        }
        if let (Some(m), Some(p)) = (mask, pend) {
            write!(f, "\t\tMasking: {:08x}  Pending: {:08x}\n", m, p)?;
        }
        Ok(())
    }
}






#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use super::*;

    #[test]
    fn display_multiview_bool() {
        assert_eq!("✓", (true).display(BoolView::CheckMark).to_string());
        assert_eq!("✗", (false).display(BoolView::CheckMark).to_string());
        assert_eq!("+", (true).display(BoolView::PlusMinus).to_string());
        assert_eq!("-", (false).display(BoolView::PlusMinus).to_string());
    }
    
    #[test]
    fn display_multiview_address() {
        let addr = Address::default();
        assert_eq!("0000:00:00.0", addr.display(Domain::Always).to_string());
        assert_eq!("00:00.0", addr.display(Domain::Suppress).to_string());
    }

    #[test]
    fn display_message_signal_interrupts() {
        use byte::{ctx::LE, BytesExt};
        let data = &include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/device/8086:2030/config"))
            [(0x60 + 2)..(0x60 + 0x18)];
        let result: MessageSignaledInterrups = data.read_with(&mut 0, LE).unwrap();
        let sample = "\
            MSI: Enable+ Count=1/2 Maskable+ 64bit-\n\
            \t\tAddress: fee00038  Data: 0000\n\
            \t\tMasking: 00000002  Pending: 00000000\n\
        ";
        assert_eq!(sample, format!("{}", result.display(Verbose(3))));
    }
}

