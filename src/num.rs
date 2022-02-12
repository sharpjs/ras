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

use std::fmt::{self, Binary, Display, Formatter, Octal, UpperHex};

// ----------------------------------------------------------------------------

/// Numeric bases.
///
/// Terminology:
///
/// ```text
/// ╭──────────────Base::Hex─────────────╮
/// │                                    │
/// significand₁₆ * power(2₁₀, exponent₁₀)
///            ╰┤         │╰┤          ├╯
///    sig_radix╯  exp_lhs╯ ╰base_radix╯
///  ∈{2,8,10,16}  ∈{2,10}      =10
/// ```
///
/// The appropriate term for `exp_lhs` is *base*, but this code avoids it to
/// prevent confusion with [`Base`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Base {
    /// Binary: *significand*₂ \[ *power*(2₁₀, *exponent*₁₀) \]
    Bin,

    /// Octal: *significand*₈ \[ *power*(2₁₀, *exponent*₁₀) \]
    Oct,

    /// Decimal: *significand*₁₀ \[ *power*(10₁₀, *exponent*₁₀) \]
    Dec,

    /// Hexadecimal: *significand*₁₆ \[ *power*(2₁₀, *exponent*₁₀) \]
    Hex,
}

impl Base {
    /// Returns the count of digits used to represent an integer or the
    /// significand in of scientific notation.
    #[inline]
    pub const fn sig_radix(self) -> u8 {
        use Base::*;
        match self {
            Bin =>  2,
            Oct =>  8,
            Dec => 10,
            Hex => 16,
        }
    }

    /// Returns the left-hand side of the exponent in scientific notation.
    ///
    /// The appropriate term is *base*, but this code avoids it to prevent
    /// confusion with [`Base`].
    #[inline]
    pub const fn exp_lhs(self) -> u8 {
        use Base::*;
        match self {
            Bin =>  2,
            Oct =>  2,
            Dec => 10,
            Hex =>  2,
        }
    }

    /// Returns the count of digits used to represent the exponent in
    /// scientific notation.
    #[inline]
    pub const fn exp_radix(self) -> u8 {
        10
    }
}

impl Base {
    /// Returns a wrapper that implements [`Display`] by formatting the given
    /// value in the base.
    #[inline]
    pub fn display<'a, T>(self, val: &'a T) -> impl Display + 'a
    where
        T: Display + Binary + Octal + UpperHex
    {
        InBase { base: self, val }
    }
}

impl Default for Base {
    #[inline]
    fn default() -> Self {
        Base::Dec
    }
}

// ----------------------------------------------------------------------------

/// Wrapper that implements [`Display`] for a value in a specific numeric base.
#[derive(Clone, Copy, Debug)]
struct InBase<'a, T>
where
    T: Display + Binary + Octal + UpperHex
{
    base: Base,
    val:  &'a T,
}

impl<T> Display for InBase<'_, T>
where
    T: Display + Binary + Octal + UpperHex
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Base::*;
        match self.base {
            Bin => write!(f, "b'{:b}", self.val),
            Oct => write!(f, "o'{:o}", self.val),
            Dec => write!(f,     "{}", self.val),
            Hex => write!(f, "x'{:X}", self.val),
        }
    }
}

// ----------------------------------------------------------------------------

/// Internal number representation.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Num {
    /// Significand.
    pub significand: u128,

    /// Exponent.
    pub exponent: i32,

    /// Significand base.
    pub base: Base,
}

impl Display for Num {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}×{}^{}",
            self.base.apply(&self.significand),
            self.base.exp_lhs(),
            self.exponent,
        )
    }
}
