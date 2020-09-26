// This file is part of ras, an assembler.
// Copyright (C) 2020 Jeffrey Sharp
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

use std::fmt::{self, Display, Formatter};

/// A source code location.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Location {
    /// The 1-based line number, or `0` to indicate an unknown line.
    pub line: u32,

    /// The 1-based column number, or `0` to indicate an unknown column.
    pub column: u32,
}

impl Location {
    pub const UNKNOWN: Self = Self::new(0, 0);
    pub const BOF:     Self = Self::new(1, 1);

    pub const fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }

    /*
    pub fn inc_line(&mut self) {
        self.line += 1
    }

    pub fn inc_column(&mut self) {
        self.column += 1
    }
    */
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match (self.line, self.column) {
            (0, 0) => Ok(()),
            (l, 0) => write!(f, "{}",    l   ),
            (l, c) => write!(f, "{}:{}", l, c),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn location_unknown() {
        assert_eq!( Location::UNKNOWN, Location::new(0, 0) );
    }

    #[test]
    fn location_bof() {
        assert_eq!( Location::BOF, Location::new(1, 1) );
    }

    #[test]
    fn location_display_fmt_0_0() {
        assert_eq!( format!("{}", Location::new(0, 0)), "" );
    }

    #[test]
    fn location_display_fmt_n_0() {
        assert_eq!( format!("{}", Location::new(1, 0)), "1" );
    }

    #[test]
    fn location_display_fmt_n_n() {
        assert_eq!( format!("{}", Location::new(1, 2)), "1:2" );
    }
}
