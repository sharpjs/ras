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

//! Integer sublexer.

use crate::lang::Base;
use super::reader::*;

pub(super) fn scan_int(input: &mut Reader, base: Base) -> (Option<u64>, usize) {
    let mut val = 0u64;     // value accumulator
    let mut len = 0;        // byte count
    let mut ovf = false;    // overflow flag

    let radix = base.radix();

    // Read until a non-digit is found
    loop {
        // Read next logical character
        let (ch, _) = input.read(&CHARS);

        // Get digit value, or 0 for separator
        // Stop when digit is greater than the radix
        let digit = ch.digit();
        if digit >= radix { break }

        // Accumulate count
        len += 1;

        // Get digit mask: 00 for separator, FF for digit
        let mask = ch.mask();

        // Accumulate digit
        let scale  = (radix ^ 1) & mask ^ 1; // 1 for separator, radix for digit
        let (v, o) = val.overflowing_mul(scale as u64); val = v; ovf |= o;
        let (v, o) = val.overflowing_add(digit as u64); val = v; ovf |= o;
    };

    // Unread the logical character that caused loop exit
    input.unread();

    (if ovf { None } else { Some(val) }, len)
}

// ----------------------------------------------------------------------------

/// Entry in the mapping of bytes to logical characters.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Char {
    Dig0, Dig1, Dig2, Dig3, Dig4, Dig5, Dig6, Dig7,
    Dig8, Dig9, DigA, DigB, DigC, DigD, DigE, DigF,
    Etc,
    Eof,
    Sep = 0b_1000_0000
}

impl LogicalChar for Char {
    const NON_ASCII: Self = Self::Etc;
    const EOF:       Self = Self::Eof;
}

impl Char {
    #[inline]
    fn digit(self) -> u8 {
        self as u8 & 0x1F
    }

    #[inline]
    fn mask(self) -> u8 {
        !((self as i8 >> 7) as u8)
    }
}

/// Mapping of 7-bit ASCII to logical characters.
static CHARS: [Char; 128] = {
    use Char::*;
    const __: Char = Etc;
[
//  x0      x1      x2      x3      x4      x5      x6      x7
//  x8      x9      xA      xB      xC      xD      xE      xF
    __,     __,     __,     __,     __,     __,     __,     __,     // 0x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 0x │·tn··r··│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 2x │ !"#$%&'│
    __,     __,     __,     __,     __,     __,     __,     __,     // 2x │()*+,-./│
    Dig0,   Dig1,   Dig2,   Dig3,   Dig4,   Dig5,   Dig6,   Dig7,   // 3x │01234567│
    Dig8,   Dig9,   __,     __,     __,     __,     __,     __,     // 3x │89:;<=>?│
    __,     DigA,   DigB,   DigC,   DigD,   DigE,   DigF,   __,     // 4x │@ABCDEFG│
    __,     __,     __,     __,     __,     __,     __,     __,     // 4x │HIJKLMNO│
    __,     __,     __,     __,     __,     __,     __,     __,     // 5x │PQRSTUVW│
    __,     __,     __,     __,     __,     __,     __,     Sep,    // 5x │XYZ[\]^_│
    __,     DigA,   DigB,   DigC,   DigD,   DigE,   DigF,   __,     // 6x │`abcdefg│
    __,     __,     __,     __,     __,     __,     __,     __,     // 6x │hijklmno│
    __,     __,     __,     __,     __,     __,     __,     __,     // 7x │pqrstuvw│
    __,     __,     __,     __,     __,     __,     __,     __,     // 7x │xyz{|}~░│
]};

#[cfg(test)]
mod tests {
    use crate::lang::Base::{self, *};
    use super::super::reader::Reader;
    use super::scan_int;

    static BASES: [Base; 4] = [Bin, Oct, Dec, Hex];

    #[test]
    fn scan_int_empty() {
        for &base in &BASES { scan_int_empty_(base) }
    }

    fn scan_int_empty_(base: Base) {
        let mut reader = Reader::new(b"");

        let (val, len) = scan_int(&mut reader, base);

        assert_eq!(val, None);
        assert_eq!(len, 0);
        assert_eq!(reader.remaining(), b"");
    }

    /*

    #[test]
    fn scan_int_junk() {
        for &base in &BASES { scan_int_junk_(base) }
    }

    fn scan_int_junk_(base: Base) {
        let mut reader = Reader::new(b"?");

        let (val, len) = scan_int(&mut reader, base);

        assert_eq!(val, 0);
        assert_eq!(len, 0);
        assert_eq!(reader.remaining(), b"?");
    }

    #[test]
    fn scan_int_zero() {
        for &base in &BASES { scan_int_zero_(base) }
    }

    fn scan_int_zero_(base: Base) {
        let mut reader = Reader::new(b"0+");

        let (val, len) = scan_int(&mut reader, base);

        assert_eq!(val, 0);
        assert_eq!(len, 1);
        assert_eq!(reader.remaining(), b"+");
    }

    #[test]
    fn scan_int_zero_eof() {
        for &base in &BASES { scan_int_zero_eof_(base) }
    }

    fn scan_int_zero_eof_(base: Base) {
        let mut reader = Reader::new(b"0");

        let (val, len) = scan_int(&mut reader, base);

        assert_eq!(val, 0);
        assert_eq!(len, 1);
        assert_eq!(reader.remaining(), b"");
    }

    #[test]
    fn scan_int_all_digits() {
        scan_int_typical_(Bin, b"01_234567_89_ABCDEFG", 0b01,                2, b"234567_89_ABCDEFG");
        scan_int_typical_(Oct, b"01_234567_89_ABCDEFG", 0o01_234567,         8,        b"89_ABCDEFG");
        scan_int_typical_(Dec, b"01_234567_89_ABCDEFG", 0_0123456789,       10,           b"ABCDEFG");
        scan_int_typical_(Hex, b"01_234567_89_ABCDEFG", 0x0123456789ABCDEF, 16,                 b"G");
        scan_int_typical_(Hex, b"01_234567_89_abcdefg", 0x0123456789abcdef, 16,                 b"g");
    }

    fn scan_int_typical_(base: Base, bytes: &[u8], v: u64, l: u8, r: &[u8]) {
        let mut reader = Reader::new(bytes);

        let (val, len) = scan_int(&mut reader, base);

        assert_eq!(val, v);
        assert_eq!(len, l);
        assert_eq!(reader.remaining(), r);
    }

    #[test]
    fn scan_int_max() {
        scan_int_max_(Bin, b"11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111+", 64);
        scan_int_max_(Oct, b"1_777_777_777_777_777_777_777+", 22);
        scan_int_max_(Dec, b"18_446_744_073_709_551_615+",    20);
        scan_int_max_(Hex, b"FFFF_FFFF_FFFF_FFFF+",           16);

    }
    fn scan_int_max_(base: Base, bytes: &[u8], exp_len: u8) {
        let mut reader = Reader::new(bytes);

        let (val, len) = scan_int(&mut reader, base);

        assert_eq!(val, 18_446_744_073_709_551_615);
        assert_eq!(len, exp_len);
        assert_eq!(reader.remaining(), b"+");
    }

    #[test]
    fn scan_int_overflow() {
        // max + 1
        scan_int_overflow_(Bin, b"1_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000+");
        scan_int_overflow_(Oct, b"2_000_000_000_000_000_000_000+");
        scan_int_overflow_(Dec, b"18_446_744_073_709_551_616+");
        scan_int_overflow_(Hex, b"1_0000_0000_0000_0000+");

        // huge
        scan_int_overflow_(Bin, b"11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111+");
        scan_int_overflow_(Oct, b"777_777_777_777_777_777_777_777_777_777_777_777_777_777+");
        scan_int_overflow_(Dec, b"999_999_999_999_999_999_999_999_999_999_999_999_999_999+");
        scan_int_overflow_(Hex, b"FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF+");
    }

    fn scan_int_overflow_(base: Base, bytes: &[u8]) {
        let mut reader = Reader::new(bytes);

        let (_, len) = scan_int(&mut reader, base);

        assert_eq!(len, 0);
        assert_eq!(reader.remaining(), b"+");
    }
    */
}
