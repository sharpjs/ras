// This file is part of ras, an assembler.
// Copyright (C) 2019 Jeffrey Sharp
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

// TODO: Just make an Eof or LogChar trait
use crate::util::ConstDefault;

// ----------------------------------------------------------------------------

/// Logical characters for lexical analysis of numeric literals.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Char {
    Non = 0,    // ..Z digit >= radix (non-base)
    Dig = 1,    // 0.. digit <  radix
    Sep = 2,    // _   separator
    Dot = 3,    // .   radix point
    Exp = 4,    // Pp  exponent mark
    Pos = 5,    // +   positive sign
    Neg = 6,    // -   negative sign
    Etc = 7,    //     other non-identifier character
    Eof = 8,    //     end of file
}

impl Char {
    /// Count of logical characters.
    const COUNT: usize = Self::Eof as usize /* / State::COUNT */ + 1;
}

impl ConstDefault for Char {
    /// Default logical character.
    /// A [`Reader`] returns this value at the end of input.
    const DEFAULT: Self = Self::Eof;
}

/// An entry in the mapping of UTF-8 bytes to logical characters.
#[derive(Clone, Copy, Debug)]
pub struct CharEntry (u8);

/// Numerical bases.
#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub enum BaseFlag {
    Bin = 63 - 4,
    Oct = 63 - 5,
    Dec = 63 - 6,
    Hex = 63 - 7
}

impl CharEntry {
    /// Returns the mask for digit accumulation.
    ///
    /// If the entry represents a digit in the given `base`, this function
    /// returns [`std::u64::MAX`].  Otherwise, this function returns `0`.
    #[inline(always)]
    pub fn mask(self, base: BaseFlag) -> u64 {
        ((self.0 as i64) << base as usize >> 63) as u64
    }

    /// Returns the digit value for digit accumulation.
    ///
    /// If the entry represents a digit in any supported base, this function
    /// returns the digit value.  Otherwise, the return value is undefined.
    #[inline(always)]
    pub fn digit(self) -> u64 {
        self.0 as u64 & 0xF
    }

    /// Returns the logical character.
    #[inline(always)]
    pub fn logical_char(self, mask: u64) -> Char {
        use std::mem::transmute;

        // Compute masks
        let is_base_digit = mask                  as u8; // 0xFF if digit in this base
        let is_some_digit = ((self.0 as i8) >> 7) as u8; // 0xFF if digit in any  base

        // Decide what logical character to return if the entry represents a
        // digit.  This will be Dig=1 if the digit is in the current base, and
        // Non=0 otherwise.
        let chr = is_base_digit & Char::Dig as u8;

        // Decide the logical character to return.  If the entry represents a
        // digit, use the value (Non or Dig) decided in the previous step.
        // Otherwise, use the entry itself as the logical character value.
        // Uses "Merge bits from two values according to a mask" hack:
        // https://graphics.stanford.edu/~seander/bithacks.html#MaskedMerge
        let chr = is_some_digit & (self.0 ^ chr) ^ self.0;

        unsafe { transmute(chr) }
    }
}

/// Mapping of UTF-8 bytes to logical characters.
static CHAR_MAP: [CharEntry; 256] = {
    use Char::*;

    // Table entry constructors:
    //                                              ┌──────────── is digit in base 16
    //                                              │┌─────────── is digit in base 10
    //                                              ││┌────────── is digit in base  8
    //                                              │││┌───────── is digit in base  2
    //                                              ││││ ┌──┬───┬ digit value
    //                                              XDOB_VVVV   V
    const fn b(v: u8)   -> CharEntry { CharEntry(0b_1111_0000 | v) } // bin digit
    const fn o(v: u8)   -> CharEntry { CharEntry(0b_1110_0000 | v) } // oct digit
    const fn d(v: u8)   -> CharEntry { CharEntry(0b_1100_0000 | v) } // dec digit
    const fn x(v: u8)   -> CharEntry { CharEntry(0b_1000_0000 | v) } // hex digit
    const fn c(c: Char) -> CharEntry { CharEntry(c as u8) }          // character
    const __:              CharEntry = c(Etc);
[
//  7-bit ASCII characters
//  x0      x1      x2      x3      x4      x5      x6      x7      CHARS
    __,     __,     __,     __,     __,     __,     __,     __,     // ........
    __,     __,     __,     __,     __,     __,     __,     __,     // .tn..r..
    __,     __,     __,     __,     __,     __,     __,     __,     // ........
    __,     __,     __,     __,     __,     __,     __,     __,     // ........
    __,     __,     __,     __,     __,     __,     __,     __,     //  !"#$%&'
    __,     __,     __,     c(Pos), __,     c(Neg), c(Dot), __,     // ()*+,-./
    b(0),   b(1),   o(2),   o(3),   o(4),   o(5),   o(6),   o(7),   // 01234567
    d(8),   d(9),   __,     __,     __,     __,     __,     __,     // 89:;<=>?
    __,     x(0xA), x(0xB), x(0xC), x(0xD), x(0xE), x(0xF), c(Non), // @ABCDEFG
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // HIJKLMNO
    c(Exp), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // PQRSTUVW
    c(Non), c(Non), c(Non), __,     __,     __,     __,     c(Sep), // XYZ[\]^_
    __,     x(0xA), x(0xB), x(0xC), x(0xD), x(0xE), x(0xF), c(Non), // `abcdefg
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // hijklmno
    c(Exp), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // pqrstuvw
    c(Non), c(Non), c(Non), __,     __,     __,     __,     __,     // xyz{|}~. <- DEL

//  UTF-8 multibyte sequences
//  0 (8)   1 (9)   2 (A)   3 (B)   4 (C)   5 (D)   6 (E)   7 (F)   RANGE
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // 80-87
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // 88-8F
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // 90-97
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // 98-9F
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // A0-A7
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // A8-AF
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // B0-B7
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // B8-BF
    __,     __,     c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // C0-C7
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // C8-CF
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // D0-D7
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // D8-DF
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // E0-E7
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // E8-EF
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // F0-F7
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), __,     __,     // F8-FF
]};

pub fn accumulate(mut acc: u64, byte: u8, base: BaseFlag) -> (Char, u64) {
    let entry = CHAR_MAP[byte as usize];
    let mask  = entry.mask(base);
    let digit = entry.digit();

    acc = (acc * 10 + digit) & mask | acc & !mask;

    let chr = entry.logical_char(mask);
    (chr, acc)
}

