use core::fmt;


use crate::device;

pub mod lspci;

// mod header;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verbose(pub usize);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiView<T, V> {
    pub data: T,
    pub view: V,
}

pub trait DisplayMultiView<'a> {
    type Data;
    type View;
    fn display(&'a self, view: Self::View) -> MultiView<Self::Data, Self::View>;
}

pub trait DisplayMultiViewBasic<V> where Self: Sized {
    fn display(&self, view: V) -> MultiView<&Self, V> {
        MultiView { data: self, view }
    }
}


/// Boolean ,ultiple view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BoolView {
    // true: ✓, false: ✗
    CheckMark,
    /// true: +, false: -
    PlusMinus,
    /// Any string
    Str(&'static str),
}

impl DisplayMultiViewBasic<BoolView> for bool {}
impl<'a> fmt::Display for MultiView<&'a bool, BoolView> {
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



#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use pcics::capabilities::MessageSignaledInterrups;
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

