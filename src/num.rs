// This file is part of ras, an assembler.
// Copyright 2022 Jeffrey Sharp
//
// SPDX-License-Identifier: GPL-3.0-or-later
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

//! Number support.

/// Numeric bases.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Base {
    /// Binary.
    Bin,

    /// Octal.
    Oct,

    /// Decimal.
    Dec,

    /// Hexadecimal.
    Hex,
}

impl Base {
    /// Returns the count of digits used to represent numbers in the base.
    #[inline]
    pub const fn radix(self) -> u8 {
        use Base::*;

        match self {
            Bin =>  2,
            Oct =>  8,
            Dec => 10,
            Hex => 16,
        }
    }
}

impl Default for Base {
    #[inline]
    fn default() -> Self {
        Base::Dec
    }
}
