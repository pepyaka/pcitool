use std::fmt;

pub mod lspci;

/// Struct that has arbitrary [fmt::Display] implementations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiView<T, V> {
    pub data: T,
    pub view: V,
}

/// Trait that has .display() method for arbitrary view types
pub trait DisplayMultiView<V>: Sized {
    fn display(&self, view: V) -> MultiView<&Self, V> {
        MultiView { data: self, view }
    }
}

/// Boolean multiple view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BoolView {
    /// true: ✓, false: ✗
    CheckMark,
    /// true: +, false: -
    PlusMinus,
    /// Any string
    Str(&'static str),
}

impl DisplayMultiView<BoolView> for bool {}
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn display_multiview_bool() {
        assert_eq!("✓", (true).display(BoolView::CheckMark).to_string());
        assert_eq!("✗", (false).display(BoolView::CheckMark).to_string());
        assert_eq!("+", (true).display(BoolView::PlusMinus).to_string());
        assert_eq!("-", (false).display(BoolView::PlusMinus).to_string());
    }
}
