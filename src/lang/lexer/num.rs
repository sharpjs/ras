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

//! Numeric literal sublexer.
//!
//! ### Number format:
//!
//! ```text
//! [base] significand [exponent]
//! ─┬──── ───┬─────── ──┬───────
//!  ├─ b'    ├─ 1       ├─ p1
//!  ├─ o'    ├─ 1.      ├─ p+1
//!  ├─ d'    ├─ 1.1     └─ p-1
//!  └─ x'    └─  .1
//! ```

use crate::lang::input::LogicalChar;
use crate::num::Base;
use super::*;

// ----------------------------------------------------------------------------

/// Logical characters recognized by the numeric literal sublexer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Char {
    // Digits
    Dig0, Dig1, Dig2, Dig3, Dig4, Dig5, Dig6, Dig7,
    Dig8, Dig9, DigA, DigB, DigC, DigD, DigE, DigF,

    // Non-digits
    Sep = non_digit(Row::Dig), // _
    Rad = non_digit(Row::Rad), // .
    Exp = non_digit(Row::Exp), // P p
    Pos = non_digit(Row::Pos), // +
    Neg = non_digit(Row::Neg), // -
    Ltr = non_digit(Row::Ltr), // G-O Q-Z g-o q-z
    Etc = non_digit(Row::Etc), // everything else
}

/// Helper to define `Char` variants.
const fn non_digit(row: Row) -> u8 {
    0x80 | row as u8
}

impl LogicalChar for Char {
    const NON_ASCII: Self = Self::Ltr;
    const EOF:       Self = Self::Etc;
}

impl Char {
    /// Returns `0` if the character is a digit; otherwise, returns `0xFF`.
    #[inline]
    const fn mask(self) -> u8 {
        // 0 if digit, 0xFF if non-digit
        (self as i8 >> 7) as u8
    }

    /// Returns the digit value if the character is a digit; otherwise, returns 0.
    #[inline]
    const fn digit(self) -> (u8, u8) {
        let mask  = !self.mask(); // 0xFF if digit, 0 if non-digit
        let digit =  self as u8 & mask;
        (digit, mask)
    }

    /// Returns the transition table row to use for the character.
    #[inline]
    const fn transition_row(self) -> Row {
        let mask = self.mask() >> 1; // 0 if digit, 0x7F if non-digit
        let row  = self as u8 & mask;
        // SAFETY: Embedded values are constrained by `non_digit()`.
        unsafe { std::mem::transmute(row) }
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
    __,     __,     __,     __,     __,     __,     __,     __,     // 0x │·tnvfr··│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 2x │ !"#$%&'│
    __,     __,     __,     Pos,    __,     Neg,    Rad,    __,     // 2x │()*+,-./│
    Dig0,   Dig1,   Dig2,   Dig3,   Dig4,   Dig5,   Dig6,   Dig7,   // 3x │01234567│
    Dig8,   Dig9,   __,     __,     __,     __,     __,     __,     // 3x │89:;<=>?│
    __,     DigA,   DigB,   DigC,   DigD,   DigE,   DigF,   Ltr,    // 4x │@ABCDEFG│
    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    // 4x │HIJKLMNO│
    Exp,    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    // 5x │PQRSTUVW│
    Ltr,    Ltr,    Ltr,    __,     __,     __,     __,     Sep,    // 5x │XYZ[\]^_│
    __,     DigA,   DigB,   DigC,   DigD,   DigE,   DigF,   Ltr,    // 6x │`abcdefg│
    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    // 6x │hijklmno│
    Exp,    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    Ltr,    // 7x │pqrstuvw│
    Ltr,    Ltr,    Ltr,    __,     __,     __,     __,     __,     // 7x │xyz{|}~░│
]};

// ----------------------------------------------------------------------------

/// Rows in the numeric literal sublexer transition table.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Row {
    Dig = row(0), // 0-9 A-F a-f _
    Rad = row(1), // .
    Exp = row(2), // P p
    Pos = row(3), // +
    Neg = row(4), // -
    Ltr = row(5), // G-O Q-Z g-o q-z
    Etc = row(6), // everything else
}

// Helper to define `Row` variants.
const fn row(n: u8) -> u8 {
    n * State::COUNT as u8
}

impl Row {
    /// Count of transition table rows.
    const COUNT: usize = Self::Etc as usize / State::COUNT + 1 ;
}

// ----------------------------------------------------------------------------

/// Numeric literal sublexer states.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    /// In the integer part.  No significand digits have been scanned.
    Int0,

    /// In the integer part.  One or more significand digits have been scanned.
    IntN,

    /// In the fraction part.  No significand digits have been scanned.
    Frac0,

    /// In the fraction part.  One or more significand digits have been scanned.
    FracN,

    /// After the exponent introducer.  Exponent sign or digits may follow.
    Exp0,

    /// In the exponent part.  No exponent digits have been scanned.
    Exp0S,

    /// In the exponent part.  One or more exponent digits have been scanned.
    ExpN,

    /// Skipping digits and letters before recording an error.
    Invalid,
}

impl State {
    /// Count of sublexer states.
    const COUNT: usize = Self::Invalid as usize + 1;

    /// Returns the amount to increment fraction precision when a digit is scanned in the state.
    fn precision_inc(self) -> u8 {
        use State::*;
        match self {
            Frac0 | FracN => 1,
            _             => 0,
        }
    }
}

// ----------------------------------------------------------------------------

/// Numeric literal sublexer actions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Action {
    /// Continue scanning.
    Continue,

    /// Store the significand and continue scanning.
    SetSignificand,

    /// Yield an integer literal or an overflow error.
    ///
    /// If no overflow, store the significand and return [`Some(Token::Int)`].
    /// Otherwise, add an overflow error and return [`None`].
    YieldInt,

    /// Yield a positional-notation floating-point literal or an overflow error.
    ///
    /// If no overflow, store the significand and return [`Some(Token::Float)`].
    /// Otherwise, add an overflow error and return [`None`].
    YieldFloat,

    /// Yield a scientific-notation floating-point literal or an overflow error.
    ///
    /// If no overflow, store the exponent and return [`Some(Token::Float)`].
    /// Otherwise, add an overflow error and return [`None`].
    YieldSci,

    /// Yield an invalid-number error.
    ///
    /// Add an invalid-token error and return [`None`].
    Error,
}

// ----------------------------------------------------------------------------

// Numeric literal sublexer transitions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Transition {
    /// Consume input and continue scanning in `IntN` state.
    IntN,

    /// Consume input and continue scanning in `Frac0` state.
    Frac0,

    /// Consume input and continue scanning in `FracN` state.
    FracN,

    /// Consume input, store significand, and continue scanning in `Exp0` state.
    Exp0,

    /// Consume input, set exponent sign to positive, and continue scanning in `Exp0S` state.
    Exp0Pos,

    /// Consume input, set exponent sign to negative, and continue scanning in `Exp0S` state.
    Exp0Neg,

    /// Consume input and continue scanning in `ExpN` state.
    ExpN,

    /// Consume input and continue scanning in `Invalid` state.
    Invalid,

    /// Yield an integer literal or an overflow error.
    ///
    /// If no overflow, store significand and return [`Some(Token::Int)`].
    /// Otherwise, add an overflow error and return [`None`].
    YInt,

    /// Yield a positional-notation floating-point literal or an overflow error.
    ///
    /// If no overflow, store significand and return [`Some(Token::Float)`].
    /// Otherwise, add an overflow error and return [`None`].
    YFloat,

    /// Yield a scientific-notation floating-point literal or an overflow error.
    ///
    /// If no overflow, store exponent and return [`Some(Token::Float)`].
    /// Otherwise, add an overflow error and return [`None`].
    YSci,

    /// Yield an invalid-number error.
    ///
    /// Add an invalid-token error and return [`None`].
    Error,
}

impl Transition {
    /// Returns a tuple consisting of the next state, the action to perform,
    /// and a flag indicating whether the exponent is negative.
    fn decode(self) -> (State, Action, bool) {
        use Action::*;
        use State      as S;
        use Transition as X;

        match self {
        //  Transition      State       Action          Inverse
            X::IntN    => ( S::IntN,    Continue,       false ),
            X::Frac0   => ( S::Frac0,   Continue,       false ),
            X::FracN   => ( S::FracN,   Continue,       false ),
            X::Exp0    => ( S::Exp0,    SetSignificand, false ),
            X::Exp0Pos => ( S::Exp0S,   Continue,       false ),
            X::Exp0Neg => ( S::Exp0S,   Continue,       true  ),
            X::ExpN    => ( S::ExpN,    Continue,       false ),
            X::Invalid => ( S::Invalid, Continue,       false ),
            X::YInt    => ( S::Invalid, YieldInt,       false ),
            X::YFloat  => ( S::Invalid, YieldFloat,     false ),
            X::YSci    => ( S::Invalid, YieldSci,       false ),
            X::Error   => ( S::Invalid, Error,          false ),
        }
    }
}

/// Numeric literal sublexer state transition table.
static TRANSITION_MAP: [Transition; Row::COUNT * State::COUNT] = {
    use Transition::*;
    const __: Transition = Invalid;
    const XX: Transition = Error;
[
  //        Int0     IntN     Frac0    FracN    Exp0     Exp0S    ExpN     Invalid
  //        <-       1<-      .<-      .1<-     1.1p<-   1.1p+<-  1.1p+1<- 1z<-
  //        -------- -------- -------- -------- -------- -------- -------- --------
  /* 0A_ */ IntN,    IntN,    FracN,   FracN,   ExpN,    ExpN,    ExpN,    __,
  /* .   */ Frac0,   FracN,   __,      __,      __,      __,      __,      __,
  /* P p */ __,      Exp0,    __,      Exp0,    __,      __,      __,      __,
  /* +   */ XX,      YInt,    XX,      YFloat,  Exp0Pos, XX,      YSci,    XX,
  /* -   */ XX,      YInt,    XX,      YFloat,  Exp0Neg, XX,      YSci,    XX,
  /* Ltr */ __,      __,      __,      __,      __,      __,      __,      __,
  /* Etc */ XX,      YInt,    XX,      YFloat,  XX,      XX,      YSci,    XX,
]};

// ----------------------------------------------------------------------------

impl<I: Iterator<Item = u8>> Lexer<I> {
    /// Attempts to scan a numeric literal.
    pub(super) fn scan_num(&mut self, base: Base) -> Option<Token> {
        use Action::*;

        // Get radix
        let mut radix = base.sig_radix();
        self.num.base = base;

        // Initialize state
        let mut state = State::Int0;    // Sublexer state
        let mut acc   = 0u128;          // Digit accumulator
        let mut prec  = 0u8;            // Fractional precision
        let mut inv   = false;          // Inversion (negative exponent) flag
        let mut over  = false;          // Overflow flag

        loop {
            // Read logical character
            let mut ch = self.input.classify(&CHARS).0;

            // Get digit value and mask
            // (digit, 0xFF) if digit, (0, 0) if non-digit
            let (digit, mask) = ch.digit();

            // Detect digit beyond current radix
            if digit >= radix { ch = Char::Ltr }

            // Get transition
            let (next_state, action, invert) =  {
                let row = ch.transition_row();
                TRANSITION_MAP[row as usize + state as usize].decode()
            };

            // Perform action specified by transition
            match action {
                Continue => (),
                SetSignificand => {
                    self.set_significand(acc);
                    radix = base.exp_radix();
                    acc   = 0;
                },
                YieldInt   if !over => return self.yield_int(acc),
                YieldFloat if !over => return self.yield_float_sig(acc,      prec),
                YieldSci   if !over => return self.yield_float_exp(acc, inv, prec),
                Error               => return self.fail_invalid_number(),
                _                   => return self.fail_overflow(),
            }

            // Accumulate digit
            let scale  = (radix ^ 1) & mask ^ 1; // radix if digit, 1 if non-digit
            let (v, o) = acc.overflowing_mul(scale as u128); acc = v; over |= o;
            let (v, o) = acc.overflowing_add(digit as u128); acc = v; over |= o;

            // Accumulate fractional precision
            prec += state.precision_inc() & mask;

            // Accumulate inversion
            inv |= invert;

            // Consume input and prepare for next character
            state = next_state;
            self.input.advance();
        }
    }

    #[inline]
    fn set_significand(&mut self, sig: u128) {
        self.num.significand = sig;
    }

    #[inline]
    fn yield_int(&mut self, sig: u128) -> Option<Token> {
        self.num.significand = sig;
        self.num.exponent    = 0;
        Some(Token::Int)
    }

    #[inline]
    fn yield_float_sig(&mut self, sig: u128, prec: u8) -> Option<Token> {
        self.num.significand = sig;
        self.num.exponent    = -(prec as i32);
        Some(Token::Float)
    }

    fn yield_float_exp(&mut self, exp: u128, inv: bool, prec: u8) -> Option<Token> {
        if let Some(exp) = convert_exp(exp, inv, prec) {
            self.num.exponent = exp;
            Some(Token::Float)
        } else {
            self.fail_overflow()
        }
    }

    fn fail_overflow(&mut self) -> Option<Token> {
        // TODO: add error
        eprintln!("ERROR: Overflow.");
        self.num = Num::default();
        None
    }

    fn fail_invalid_number(&mut self) -> Option<Token> {
        // TODO: add error
        eprintln!("ERROR: Invalid numeric literal.");
        self.num = Num::default();
        None
    }
}

#[inline]
fn convert_exp(exp: u128, inv: bool, prec: u8) -> Option<i32> {
    let mut exp = i32::try_from(exp).ok()?;

    // Conditional negation
    // https://graphics.stanford.edu/~seander/bithacks.html#ConditionalNegate
    let inv = inv as i32; // 0 or 1
    exp = (exp ^ -inv).checked_add(inv)?;

    exp.checked_sub(prec as i32)
}

#[cfg(test)]
mod tests {
    use crate::num::Base::{self, *};
    use super::super::*;

    static BASES: [Base; 4] = [Bin, Oct, Dec, Hex];

    #[test]
    fn scan_num_junk() {
        for text in ["+", "-", "?", ""] {
            for &base in &BASES { scan_num_junk_(text, base) }
        }
    }

    fn scan_num_junk_(text: &str, base: Base) {
        let mut lexer = Lexer::new(text.bytes());

        let result = lexer.scan_num(base);

        assert_eq!(result, None);
        assert_eq!(lexer.input.position(), 0);
    }

    #[test]
    fn scan_num_zero() {
        for text in ["0+", "0-", "0?", "0"] {
            for &base in &BASES { scan_num_zero_(text, base) }
        }
    }

    fn scan_num_zero_(text: &str, base: Base) {
        let mut lexer = Lexer::new(text.bytes());

        let result = lexer.scan_num(base);

        assert_eq!(result,                 Some(Token::Int));
        assert_eq!(lexer.input.position(), 1);
        assert_eq!(lexer.num.significand,  0);
        assert_eq!(lexer.num.exponent,     0);
        assert_eq!(lexer.num.base,         base);
    }

    /*
    // TODO: Convert and expand these tests.

    #[test]
    fn scan_num_all_digits() {
        scan_num_typical_(Bin, b"01_234567_89_ABCDEFG", 0b01,                3, b"234567_89_ABCDEFG");
        scan_num_typical_(Oct, b"01_234567_89_ABCDEFG", 0o01_234567,        10,        b"89_ABCDEFG");
        scan_num_typical_(Dec, b"01_234567_89_ABCDEFG", 0_0123456789,       13,           b"ABCDEFG");
        scan_num_typical_(Hex, b"01_234567_89_ABCDEFG", 0x0123456789ABCDEF, 19,                 b"G");
        scan_num_typical_(Hex, b"01_234567_89_abcdefg", 0x0123456789abcdef, 19,                 b"g");
    }

    fn scan_num_typical_(base: Base, bytes: &[u8], v: u64, l: usize, r: &[u8]) {
        let mut reader = Reader::new(bytes);

        let (val, len) = scan_num(&mut reader, base);

        assert_eq!(val, Some(v));
        assert_eq!(len, l);
        assert_eq!(reader.remaining(), r);
    }

    #[test]
    fn scan_num_max() {
        scan_num_max_(Bin, b"11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111+", 71);
        scan_num_max_(Oct, b"1_777_777_777_777_777_777_777+", 29);
        scan_num_max_(Dec, b"18_446_744_073_709_551_615+",    26);
        scan_num_max_(Hex, b"FFFF_FFFF_FFFF_FFFF+",           19);

    }
    fn scan_num_max_(base: Base, bytes: &[u8], exp_len: usize) {
        let mut reader = Reader::new(bytes);

        let (val, len) = scan_num(&mut reader, base);

        assert_eq!(val, Some(18_446_744_073_709_551_615));
        assert_eq!(len, exp_len);
        assert_eq!(reader.remaining(), b"+");
    }

    #[test]
    fn scan_num_overflow() {
        // max + 1
        scan_num_overflow_(Bin, b"1_00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000+", 73);
        scan_num_overflow_(Oct, b"2_000_000_000_000_000_000_000+", 29);
        scan_num_overflow_(Dec, b"18_446_744_073_709_551_616+",    26);
        scan_num_overflow_(Hex, b"1_0000_0000_0000_0000+",         21);

        // huge
        scan_num_overflow_(Bin, b"11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111+", 80);
        scan_num_overflow_(Oct, b"777_777_777_777_777_777_777_777_777_777_777_777_777_777+", 55);
        scan_num_overflow_(Dec, b"999_999_999_999_999_999_999_999_999_999_999_999_999_999+", 55);
        scan_num_overflow_(Hex, b"FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF+",  54);
    }

    fn scan_num_overflow_(base: Base, bytes: &[u8], exp_len: usize) {
        let mut reader = Reader::new(bytes);

        let (val, len) = scan_num(&mut reader, base);

        assert_eq!(val, None);
        assert_eq!(len, exp_len);
        assert_eq!(reader.remaining(), b"+");
    }
    */
}
