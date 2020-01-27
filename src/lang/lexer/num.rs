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

use super::reader::*;

fn next_num(input: &mut Reader, base: BaseFlag) {
    use super::num::Action::*;

    let mut state = State::Int0 as u8;
    let mut acc   = 0;
    let mut sig   = 0;
    let mut exp   = 0;
    let mut _fw   = 0; // TODO: fraction width

    loop {
        let (entry, _) = input.next(&CHAR_MAP);
        let mask       = entry.mask(base);
        let digit      = entry.digit();

        acc = (acc * 10 + digit) & mask | acc & !mask;
        //         ^^^^ TODO

        let chr  = entry.logical_char(mask);
        let next = TRANSITION_MAP[state as usize + chr as usize];
        let next = TRANSITION_LUT[next  as usize];

        state ^= (state ^ next.state as u8) & next.state_mask();
        sig   ^= (sig   ^ acc)              & next.sig_mask();
        exp   ^= (exp   ^ acc)              & next.exp_mask();
        exp   |=                              next.exp_sign();
        _fw   +=                              next.frac_width();

        match next.action {
            Continue => continue,
            YieldNum => {
                if chr != Char::Eof {
                    input.rewind();
                }
                return // numeric literal
            },
            YieldErr => {
                if chr != Char::Eof {
                    input.rewind();
                }
                return // error
            },
            Panic => {
                panic!()
            },
        };
    }
}

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
    const COUNT: usize = Self::Eof as usize + 1;
}

impl LogChar for Char {
    const EXT: Self = Self::Non;
    const EOF: Self = Self::Eof;
}

/// Numerical bases.
#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub enum BaseFlag {
    Bin = 63 - 4,
    Oct = 63 - 5,
    Dec = 63 - 6,
    Hex = 63 - 7
}

/// Entry in the mapping of bytes to logical characters.
#[derive(Clone, Copy, Debug)]
pub struct CharEntry (u8);

impl LogChar for CharEntry {
    const EXT: Self = CharEntry(Char::Non as u8);
    const EOF: Self = CharEntry(Char::Eof as u8);
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

/// Mapping of 7-bit ASCII to logical characters.
static CHAR_MAP: [CharEntry; 128] = {
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
//  xx0     xx1     xx2     xx3     xx4     xx5     xx6     xx7
    __,     __,     __,     __,     __,     __,     __,     __,     // 00x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 01x │·tn··r··│
    __,     __,     __,     __,     __,     __,     __,     __,     // 02x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 03x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 04x │ !"#$%&'│
    __,     __,     __,     c(Pos), __,     c(Neg), c(Dot), __,     // 05x │()*+,-./│
    b(0),   b(1),   o(2),   o(3),   o(4),   o(5),   o(6),   o(7),   // 06x │01234567│
    d(8),   d(9),   __,     __,     __,     __,     __,     __,     // 07x │89:;<=>?│
    __,     x(0xA), x(0xB), x(0xC), x(0xD), x(0xE), x(0xF), c(Non), // 10x │@ABCDEFG│
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // 11x │HIJKLMNO│
    c(Exp), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // 12x │PQRSTUVW│
    c(Non), c(Non), c(Non), __,     __,     __,     __,     c(Sep), // 13x │XYZ[\]^_│
    __,     x(0xA), x(0xB), x(0xC), x(0xD), x(0xE), x(0xF), c(Non), // 14x │`abcdefg│
    c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // 15x │hijklmno│
    c(Exp), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), c(Non), // 16x │pqrstuvw│
    c(Non), c(Non), c(Non), __,     __,     __,     __,     __,     // 17x │xyz{|}~·│
]};

// ----------------------------------------------------------------------------

// Helper to define state variants
const fn state(n: u8) -> u8 {
    n * Char::COUNT as u8
}

/// States for lexical analysis of numeric literals.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    /// At the first character of the integer part.
    Int0    = state(0),

    /// At a non-first character of the integer part.
    Int     = state(1),

    /// At the first character of the fraction part.
    Frac0   = state(2),

    /// At a non-first character of the fraction part.
    Frac    = state(3),

    /// At the first character of the exponent part.
    Exp0    = state(4),

    /// At the first non-sign character of the exponent part.
    ExpS0   = state(5),

    /// At a non-first, non-sign character of the exponent part.
    Exp     = state(6),

    /// Consuming identifier characters after an invalid numeric literal.
    Invalid = state(7),
}

impl State {
    /// Count of states.
    const COUNT: usize = State::Invalid as usize / Char::COUNT + 1;
}

// ----------------------------------------------------------------------------

/// Actions for lexical analysis of numeric literals.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Action {
    /// Continue scanning.
    Continue,

    /// Yield a numeric literal token.
    YieldNum,

    /// Yield an 'invalid numeric literal' lexical error.
    YieldErr,

    /// Panic: the lexer is in an invalid state.
    Panic,
}

// ----------------------------------------------------------------------------

// IDs of transitions in lexical analysis of numeric literals.  Each ID is an
// index into [`TRANSITION_LUT`], which contains the details of the transition.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum TransitionId {
    /// Continue scanning.
    None,

    /// Transition to `Int` state.
    Int,

    /// Transition to `Frac0` state.
    Frac0,

    /// Transition to `Frac` state.
    Frac,

    /// Transition to `Exp0` state.
    Exp0,

    /// Transition to `ExpS0` state.  Set exponent sign to positive.
    ExpP0,

    /// Transition to `ExpS0` state.  Set exponent sign to negative.
    ExpN0,

    /// Transition to `Exp` state.
    Exp,

    /// Transition to `Invalid` state.
    Inval,

    /// Yield a numeric literal, assigning accumulator to significand.
    YNumS,

    /// Yield a numeric literal, assigning accumulator to exponent.
    YNumE,

    /// Yield an 'invalid numeric literal' lexical error.
    YErr,

    /// Panic: the lexer is in an invalid state.
    Panic,
}

impl TransitionId {
    /// Count of transition IDs.
    const COUNT: usize = Self::Panic as usize + 1;
}

/// Transition in lexical analysis of numeric literals.
#[derive(Clone, Copy, Debug)]
struct Transition {
    state:  State,
    action: Action,
    flags:  u8, // 0b_CES_xxx_NW
                //    │││     │└─ increment fraction width
                //    │││     └── set exponent sign
                //    ││└──────── store accumulator to significand
                //    │└───────── store accumulator to exponent
                //    └────────── change state
}

impl Transition {
    /// Returns the increment to fraction width.
    #[inline(always)]
    fn frac_width(&self) -> u8 {
        self.flags & 1
    }

    /// Returns 1 if the exponent is signed and 0 otherwise.
    #[inline(always)]
    fn exp_sign(&self) -> u64 {
        self.flags as u64 >> 1 << 63
    }

    /// Returns [`std::u64::MAX`] if the accumulator should be stored to the
    /// significand and `0` otherwise.
    #[inline(always)]
    fn sig_mask(&self) -> u64 {
        ((self.flags as i64) << (63 - 5) >> 63) as u64
    }

    /// Returns [`std::u64::MAX`] if the accumulator should be stored to the
    /// exponent and `0` otherwise.
    #[inline(always)]
    fn exp_mask(&self) -> u64 {
        ((self.flags as i64) << (63 - 6) >> 63) as u64
    }

    /// Returns [`std::u8::MAX`] if the state should change and `0` otherwise.
    #[inline(always)]
    fn state_mask(&self) -> u8 {
        ((self.flags as i8) >> 7) as u8
    }
}

/// Lexer transitions in order by transition ID.
static TRANSITION_LUT: [Transition; TransitionId::COUNT] = {
use Action::*; use State::*; [
//                                         increment fraction width ──────────┐
//                                                set exponent sign ─────────┐│
//                                                store significand ───┐     ││
//                                                   store exponent ──┐│     ││
//                                                     change state ─┐││     ││
// Id                           State            Action              │││     ││
// -----                        --------         ---------        ---CES-----NW
/* None  */ Transition { state: Invalid, action: Continue, flags: 0b_000_000_00 },
/* Int   */ Transition { state: Int,     action: Continue, flags: 0b_100_000_00 },
/* Frac0 */ Transition { state: Frac0,   action: Continue, flags: 0b_100_000_00 },
/* Frac  */ Transition { state: Frac,    action: Continue, flags: 0b_100_000_01 },
/* Exp0  */ Transition { state: Exp0,    action: Continue, flags: 0b_101_000_00 },
/* ExpP0 */ Transition { state: ExpS0,   action: Continue, flags: 0b_100_000_00 },
/* ExpN0 */ Transition { state: ExpS0,   action: Continue, flags: 0b_100_000_10 },
/* Exp   */ Transition { state: Exp,     action: Continue, flags: 0b_100_000_00 },
/* Inval */ Transition { state: Invalid, action: Continue, flags: 0b_100_000_00 },
/* YNumS */ Transition { state: Invalid, action: YieldNum, flags: 0b_001_000_00 },
/* YNumE */ Transition { state: Invalid, action: YieldNum, flags: 0b_010_000_00 },
/* YErr  */ Transition { state: Invalid, action: YieldErr, flags: 0b_000_000_00 },
/* Panic */ Transition { state: Invalid, action: Panic,    flags: 0b_000_000_00 },
]};

/// Lexer state transition map for numeric literals.
static TRANSITION_MAP: [TransitionId; State::COUNT * Char::COUNT] = {
use TransitionId::*; [
//          ----Digits----
// State    Other   Base    _       .       Pp      +       -       etc     EOF
// -----    ------- ------- ------- ------- ------- ------- ------- ------- -------
/* Int0  */ Inval,  Int,    None,   Frac0,  Inval,  YErr,   YErr,   YErr,   YErr,
/* Int   */ Inval,  None,   None,   Frac0,  Exp0,   YNumS,  YNumS,  YNumS,  YNumS,
/* Frac0 */ Inval,  Frac,   None,   Inval,  Exp0,   YNumS,  YNumS,  YNumS,  YNumS,
/* Frac  */ Inval,  None,   None,   Inval,  Exp0,   YNumS,  YNumS,  YNumS,  YNumS,
/* Exp0  */ Inval,  Exp,    None,   Inval,  Inval,  ExpP0,  ExpN0,  YErr,   YErr,
/* ExpS0 */ Inval,  Exp,    None,   Inval,  Inval,  YNumE,  YNumE,  YNumE,  YNumE,
/* Exp   */ Inval,  None,   None,   Inval,  Inval,  YNumE,  YNumE,  YNumE,  YNumE,
/* Inval */ None,   None,   None,   None,   None,   YErr,   YErr,   YErr,   YErr,
]};

