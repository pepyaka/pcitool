use std::fmt;

use pcics::extended_capabilities::{
    resizable_bar::{ResizableBarControl, ResizableBarEntry, ResizableBarError},
    ResizableBar,
};

pub struct View<'a> {
    pub result: Result<&'a ResizableBar<'a>, &'a ResizableBarError>,
    pub verbose: usize,
    pub is_virtual: bool,
}

impl<'a> View<'a> {
    fn fmt_ok(f: &mut fmt::Formatter<'_>, rebar: &'a ResizableBar<'a>) -> fmt::Result {
        for entry in rebar.clone() {
            let ResizableBarControl {
                bar_index,
                bar_size,
                ..
            } = entry.control;
            let current_size = ResizableBarEntry::BAR_SIZES
                .get(bar_size as usize)
                .cloned()
                .unwrap_or("<unknown>");
            write!(
                f,
                "\t\tBAR {}: current size: {}, supported:",
                bar_index, current_size
            )?;
            for (n, s) in ResizableBarEntry::BAR_SIZES.iter().enumerate() {
                if entry.is_function_supports_power_of_two(n + 20) {
                    write!(f, " {}", s)?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
    fn fmt_err(f: &mut fmt::Formatter<'_>, err: &'a ResizableBarError) -> fmt::Result {
        match err {
            ResizableBarError::NumberOfResizableBars { value } => writeln!(
                f,
                "\t\t<error in resizable BAR: num_bars={} is out of specification>",
                value
            ),
            _ => writeln!(f, "\t\t<unreadable>"),
        }
    }
}

impl<'a> fmt::Display for View<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &View {
            result,
            verbose,
            is_virtual,
        } = self;
        writeln!(
            f,
            "{} Resizable BAR",
            if is_virtual { "Virtual" } else { "Physical" }
        )?;
        if verbose < 2 {
            return Ok(());
        }
        match result {
            Ok(rebar) => Self::fmt_ok(f, rebar),
            Err(err) => Self::fmt_err(f, err),
        }
    }
}
