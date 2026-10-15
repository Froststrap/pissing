//! debug printing, deliberately kept out of the core.

use std::fmt::Display;

/// `println!` anything that formats. Debug convenience only, not for serialized output.
pub trait Print {
    fn print(&self);
}

impl<T: Display> Print for T {
    fn print(&self) {
        println!("{self}");
    }
}
