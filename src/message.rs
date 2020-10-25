//! Assembler Messages
//
// This file is part of ras, an assembler.
// Copyright 2020 Jeffrey Sharp
//
// ras is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published
// by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
//
// ras is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See
// the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ras.  If not, see <http://www.gnu.org/licenses/>.

use std::fmt::{self, Arguments, Display, Formatter};
use crate::util::Location;

// -----------------------------------------------------------------------------

/// An assembler message.
#[derive(Copy, Clone, Debug)]
pub struct Message<'a> {
    /// Severity of the message.
    pub severity: Severity,

    /// Path of a source file related to the message, or the program name if no
    /// source file is related.
    pub source: &'a str,

    /// Textual location within source file related to the message, or
    /// [`Location::Unknown`] if no location is related.
    pub location: Location,

    /// Message content.
    content: Arguments<'a>
}

impl<'a> Message<'a> {
    /// Creates a `Message` with the given severity and format arguments,
    /// without a related source file path or textual location.
    #[inline]
    pub const fn new(sev: Severity, args: Arguments<'a>) -> Self {
        Self::at(crate::PROGRAM_NAME, Location::UNKNOWN, sev, args)
    }

    /// Creates a `Message` with the given severity and format arguments,
    /// related to the given source file path and textual location.
    #[inline]
    pub const fn at(path: &'a str, loc: Location, sev: Severity, args: Arguments<'a>) -> Self {
        Self {
            severity: sev,
            source:   path,
            location: loc,
            content:  args
        }
    }
}

// Display is used when a Message is printed as output.
impl Display for Message<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{}: {}{}",
            self.source,
            self.location,
            self.severity,
            self.content
        )
    }
}
 
// -----------------------------------------------------------------------------

/// Message severity levels.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Severity {
    /// For informational messages.
    Normal,

    /// For potential problems that do not prevent complete assembly.
    /// Assembly continues, and the assembler will produce output.
    Warning,

    /// For problems that prevent complete assembly.
    /// Assembly might continue, but the assembler will not produce output.
    Error,

    /// For severe, unrecoverable problems.
    /// The assembler terminates immediately and does not produce output.
    Fatal,
}

// Display is used when a Severity is printed in an assembler message.
impl Display for Severity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Severity::Normal  => "",
            Severity::Warning => "warning: ",
            Severity::Error   => "error: ",
            Severity::Fatal   => "fatal: ",
        })
    }
}

// -----------------------------------------------------------------------------

/// Creates a 'file not found' message.
pub fn file_not_found_error(path: &str) -> Message {
    Message::at(path, Location::UNKNOWN, Severity::Error, format_args!(
        "file not found"
    ))
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_not_found_error() {
        assert_eq!(
            format!("{}", file_not_found_error("foo.s")),
            "foo.s: error: file not found"
        )
    }
}
