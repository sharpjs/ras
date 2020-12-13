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

// Number format:
//
// [base] significand [exponent]
//
// where:
//   base  sig  exp
//    b'   1    p1
//    o'   1.   p+1
//    d'    .1  p-1
//    x'   1.1

use crate::lang::Base;
use super::reader::*;

#[derive(Debug)]
struct NumData {
    parts:  [(u64, usize); 3],  // integer, fraction, exponent * (value, size)
    invert: bool,               // whether exponent is negative
    base:   Base,               // base
}

impl NumData {
    fn new(base: Base) -> Self {
        Self {
            parts:  [(0, 0), (0, 0), (0, 0)],
            invert: false,
            base
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct State(u8);

static STATES: [State; 4] = [
    State(0b_011_000_00), // 0: significand integral part
    State(0b_010_001_01), // 1: significand fractional part
    State(0b_100_010_10), // 2: exponent or sign
    State(0b_000_100_10), // 3: exponent
    //      :---:---:--:
    //      |   |   |xx|  // part index: 0=integer, 1=fraction, 2=exponent
    //      |   |   | x|  // digits count toward: 0=nothing, 1=fraction scale
    //      |   |   |x |  // digits count toward: 0=significand, 1=exponent
    //      :---:---:--:
    //      |   |  x|  |  // is_state_1
    //      |   | x |  |  // is_state_2
    //      |   |x  |  |  // is_state_3 / has_exponent_sign
    //      :---:---:--:
    //      |  x|   |  |  // can_goto_state_1
    //      | x |   |  |  // can_goto_state_2
    //      |x  |   |  |  // can_goto_state_3
];

impl State {
    #[inline]
    const fn part_index(self) -> usize {
        (self.0 & 0b11) as usize
    }

    #[inline]
    const fn scale_mask(self) -> usize {
        self.mask_from_bit(0)
    }

    #[inline]
    const fn sig_digit_mask(self) -> usize {
        !self.exp_digit_mask()
    }

    #[inline]
    const fn exp_digit_mask(self) -> usize {
        self.mask_from_bit(1)
    }

    #[inline]
    const fn has_exp_sign(self) -> bool {
        self.0 & 0b100_00 != 0
    }

    #[inline]
    const fn can_transition_to(self, next: State) -> bool {
        (self.0 >> 5) & (next.0 >> 2) != 0
    }

    #[inline]
    const fn mask_from_bit(self, n: usize) -> usize {
        const USIZE_BITS: usize = std::mem::size_of::<usize>() * 8;

        ( (self.0 as isize) << (USIZE_BITS - 1 - n) >> (USIZE_BITS - 1) ) as usize
    }
}

// TODO: WIP converting this into a general number scanner
pub fn scan_int(input: &mut Reader, base: Base) -> (Option<u64>, usize) {
    let start = input.position();
    let radix = base.radix();

    let mut data  = NumData::new(base);
    let mut state = STATES[0];
    let mut ovf   = false;      // overflow flag

    loop {
        let mut val = 0u64;     // value accumulator
        let mut len = 0;        // digit count

        // Read until a non-digit is found
        let ch = loop {
            // Read next logical character
            let (ch, _) = input.read(&CHARS);

            // Get digit value, or 0 for separator
            // Stop when digit is greater than the radix
            let digit = ch.digit();
            if digit >= radix { break ch }

            // Get digit mask: 00 for separator, FF for digit
            let mask = ch.digit_mask();

            // Accumulate digit
            let scale  = (radix ^ 1) & mask ^ 1; // 1 for separator, radix for digit
            let (v, o) = val.overflowing_mul(scale as u64); val = v; ovf |= o;
            let (v, o) = val.overflowing_add(digit as u64); val = v; ovf |= o;

            // Accumulate count of digits, needed for fraction scale
            len += (1 & mask) as usize;
        };

        data.parts[state.part_index()] = (val, len);

        let next = STATES[ch.next_state() as usize];
        if !state.can_transition_to(next) { break }
        state = next;

        data.invert |= ch.invert();
    }

    // Unread the logical character that caused loop exit
    input.unread();

    ovf |= data.parts[0].1 == 0;

    (if ovf { None } else { Some(data.parts[0].0) }, input.position() - start)
}

// ----------------------------------------------------------------------------

/// Entry in the mapping of bytes to logical characters.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Char {
    // Digits
    Dig0, Dig1, Dig2, Dig3, Dig4, Dig5, Dig6, Dig7,
    Dig8, Dig9, DigA, DigB, DigC, DigD, DigE, DigF,
    // Non-digits
    //            x-- OR into sign bit
    //            | xx new state (if > current state)
    Sep = 0b_1000_0000, // separator        [_]
    Etc = 0b_1001_0000, // everything else  .|\z
    Rad = 0b_1001_0001, // radix point      [.]
    Exp = 0b_1001_0010, // exponent         [Pp]
    Pos = 0b_1001_0011, // positive sign    [+]
    Neg = 0b_1001_1011, // negative sign    [-]
}

impl LogicalChar for Char {
    const NON_ASCII: Self = Self::Etc;
    const EOF:       Self = Self::Etc;
}

impl Char {
    #[inline]
    const fn digit(self) -> u8 {
        self as u8 & 0b_1_1111
    }

    #[inline]
    const fn digit_mask(self) -> u8 {
        !self.state_mask()
    }

    #[inline]
    const fn state_mask(self) -> u8 {
        (self as i8 >> 7) as u8
    }

    #[inline]
    const fn next_state(self) -> u8 {
        self as u8 & self.state_mask() & 0b11
    }

    #[inline]
    const fn invert(self) -> bool {
        self as u8 & 0b1000 != 0
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
    __,     __,     __,     Pos,    __,     Neg,    Rad,    __,     // 2x │()*+,-./│
    Dig0,   Dig1,   Dig2,   Dig3,   Dig4,   Dig5,   Dig6,   Dig7,   // 3x │01234567│
    Dig8,   Dig9,   __,     __,     __,     __,     __,     __,     // 3x │89:;<=>?│
    __,     DigA,   DigB,   DigC,   DigD,   DigE,   DigF,   __,     // 4x │@ABCDEFG│
    __,     __,     __,     __,     __,     __,     __,     __,     // 4x │HIJKLMNO│
    Exp,    __,     __,     __,     __,     __,     __,     __,     // 5x │PQRSTUVW│
    __,     __,     __,     __,     __,     __,     __,     Sep,    // 5x │XYZ[\]^_│
    __,     DigA,   DigB,   DigC,   DigD,   DigE,   DigF,   __,     // 6x │`abcdefg│
    __,     __,     __,     __,     __,     __,     __,     __,     // 6x │hijklmno│
    Exp,    __,     __,     __,     __,     __,     __,     __,     // 7x │pqrstuvw│
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

    #[test]
    fn scan_int_junk() {
        for &base in &BASES { scan_int_junk_(base) }
    }

    fn scan_int_junk_(base: Base) {
        let mut reader = Reader::new(b"?");

        let (val, len) = scan_int(&mut reader, base);

        assert_eq!(val, None);
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

        assert_eq!(val, Some(0));
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

        assert_eq!(val, Some(0));
        assert_eq!(len, 1);
        assert_eq!(reader.remaining(), b"");
    }

    #[test]
    fn scan_int_all_digits() {
        scan_int_typical_(Bin, b"01_234567_89_ABCDEFG", 0b01,                3, b"234567_89_ABCDEFG");
        scan_int_typical_(Oct, b"01_234567_89_ABCDEFG", 0o01_234567,        10,        b"89_ABCDEFG");
        scan_int_typical_(Dec, b"01_234567_89_ABCDEFG", 0_0123456789,       13,           b"ABCDEFG");
        scan_int_typical_(Hex, b"01_234567_89_ABCDEFG", 0x0123456789ABCDEF, 19,                 b"G");
        scan_int_typical_(Hex, b"01_234567_89_abcdefg", 0x0123456789abcdef, 19,                 b"g");
    }

    fn scan_int_typical_(base: Base, bytes: &[u8], v: u64, l: usize, r: &[u8]) {
        let mut reader = Reader::new(bytes);

        let (val, len) = scan_int(&mut reader, base);

        assert_eq!(val, Some(v));
        assert_eq!(len, l);
        assert_eq!(reader.remaining(), r);
    }

    #[test]
    fn scan_int_max() {
        scan_int_max_(Bin, b"11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111+", 71);
        scan_int_max_(Oct, b"1_777_777_777_777_777_777_777+", 29);
        scan_int_max_(Dec, b"18_446_744_073_709_551_615+",    26);
        scan_int_max_(Hex, b"FFFF_FFFF_FFFF_FFFF+",           19);

    }
    fn scan_int_max_(base: Base, bytes: &[u8], exp_len: usize) {
        let mut reader = Reader::new(bytes);

        let (val, len) = scan_int(&mut reader, base);

        assert_eq!(val, Some(18_446_744_073_709_551_615));
        assert_eq!(len, exp_len);
        assert_eq!(reader.remaining(), b"+");
    }

    #[test]
    fn scan_int_overflow() {
        // max + 1
        scan_int_overflow_(Bin, b"1_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000+", 73);
        scan_int_overflow_(Oct, b"2_000_000_000_000_000_000_000+", 29);
        scan_int_overflow_(Dec, b"18_446_744_073_709_551_616+",    26);
        scan_int_overflow_(Hex, b"1_0000_0000_0000_0000+",         21);

        // huge
        scan_int_overflow_(Bin, b"11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111+", 80);
        scan_int_overflow_(Oct, b"777_777_777_777_777_777_777_777_777_777_777_777_777_777+", 55);
        scan_int_overflow_(Dec, b"999_999_999_999_999_999_999_999_999_999_999_999_999_999+", 55);
        scan_int_overflow_(Hex, b"FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF+",  54);
    }

    fn scan_int_overflow_(base: Base, bytes: &[u8], exp_len: usize) {
        let mut reader = Reader::new(bytes);

        let (val, len) = scan_int(&mut reader, base);

        assert_eq!(val, None);
        assert_eq!(len, exp_len);
        assert_eq!(reader.remaining(), b"+");
    }
}
