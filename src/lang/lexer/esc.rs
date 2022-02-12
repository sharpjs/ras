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

//! Escape sequence sublexer.
//!
//! ### Escape Sequences
//!
//! Sequence | UTF-8   | Name  | Description
//! ---------|---------|:------|:-----------
//! `\0`     | `00`    | `NUL` | null character
//! `\a`     | `07`    | `BEL` | bell, alert
//! `\b`     | `08`    | `BS`  | backspace
//! `\t`     | `09`    | `HT`  | horizontal tab
//! `\n`     | `0A`    | `LF`  | line feed, newline
//! `\v`     | `0B`    | `VT`  | vertical tab
//! `\f`     | `0C`    | `FF`  | form feed
//! `\r`     | `0D`    | `CR`  | carriage return
//! `\e`     | `1B`    | `ESC` | escape
//! `\s`     | `20`    | ` `   | space
//! `\"`     | `22`    | `"`   | double quote
//! `\'`     | `27`    | `'`   | single quote
//! `\\`     | `5C`    | `\`   | backslash
//! `\d`     | `7F`    | `DEL` | delete

use crate::lang::input::LogicalChar;
use super::*;

type Result = std::result::Result<(), ()>;

// ----------------------------------------------------------------------------

/// Logical characters recognized by the escape sequence sublexer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Char {
    Digit  = char( 0), // 0-9
    HexUc  = char( 1), // A-F
    LtrUc  = char( 2), // G-Z except U X
    HexLc  = char( 3), // a-f
    LtrLc  = char( 4), // g-z except u x
    LtrU   = char( 5), // U u
    LtrX   = char( 6), // X x
    Pass   = char( 7), // "'\
    LCurly = char( 8), // {
    RCurly = char( 9), // }
    Other  = char(10), // everything else
    Eof    = char(11), // end of file // <- COUNT references this
}

// Helper to define Char variants.
const fn char(n: u8) -> u8 {
    n * State::COUNT as u8
}

impl Char {
    /// Count of logical characters.
    const COUNT: usize = Self::Eof as usize / State::COUNT as usize + 1;
}

impl LogicalChar for Char {
    const NON_ASCII: Self = Self::Other;
    const EOF:       Self = Self::Eof;
}

/// Mapping of 7-bit ASCII to logical characters.
static CHARS: [Char; 128] = {
    use Char::*;
    const __: Char = Other;
[
//  x0      x1      x2      x3      x4      x5      x6      x7
//  x8      x9      xA      xB      xC      xD      xE      xF
    __,     __,     __,     __,     __,     __,     __,     __,     // 0x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 0x │·tnvfr··│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    __,     __,     Pass,   __,     __,     __,     __,     Pass,   // 2x │ !"#$%&'│
    __,     __,     __,     __,     __,     __,     __,     __,     // 2x │()*+,-./│
    Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  // 3x │01234567│
    Digit,  Digit,  __,     __,     __,     __,     __,     __,     // 3x │89:;<=>?│
    __,     HexUc,  HexUc,  HexUc,  HexUc,  HexUc,  HexUc,  LtrUc,  // 4x │@ABCDEFG│
    LtrUc,  LtrUc,  LtrUc,  LtrUc,  LtrUc,  LtrUc,  LtrUc,  LtrUc,  // 4x │HIJKLMNO│
    LtrUc,  LtrUc,  LtrUc,  LtrUc,  LtrUc,  LtrU,   LtrUc,  LtrUc,  // 5x │PQRSTUVW│
    LtrX,   LtrUc,  LtrUc,  __,     Pass,   __,     __,     __,     // 5x │XYZ[\]^_│
    __,     HexLc,  HexLc,  HexLc,  HexLc,  HexLc,  HexLc,  LtrLc,  // 6x │`abcdefg│
    LtrLc,  LtrLc,  LtrLc,  LtrLc,  LtrLc,  LtrLc,  LtrLc,  LtrLc,  // 6x │hijklmno│
    LtrLc,  LtrLc,  LtrLc,  LtrLc,  LtrLc,  LtrU,   LtrLc,  LtrLc,  // 7x │pqrstuvw│
    LtrX,   LtrLc,  LtrLc,  LCurly, __,     RCurly, __,     __,     // 7x │xyz{|}~░│
]};

// ----------------------------------------------------------------------------

/// Escape sequence sublexer states.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    /// Initial state, following `\`.
    Esc,

    /// Expecting the first hex digit after `\x`
    X0,

    /// Expecting the second hex digit after `\x`
    X1,

    /// Expecting the `{` after `\u`.
    U,

    /// Expecting the first hex digit after `\u{`.
    U0,

    /// Expecting another hex digit after `\u{`.
    UN, // <- COUNT references this
}

impl State {
    /// Count of sublexer states.
    const COUNT: usize = Self::UN as usize + 1;
}

// ----------------------------------------------------------------------------

// Escape sequence sublexer transitions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Transition {
    /// Complete a verbatim escape.
    Pass,

    /// Complete a simple escape where the input byte is a decimal digit.
    Dig,

    /// Complete a simple escape where the input byte is an upper-case letter.
    Uc,

    /// Complete a simple escape where the input byte is a lower-case letter.
    Lc,

    /// Transition to [`State::X0`] and continue scanning.
    X0,

    /// Accumulate a decimal digit.  Transition to [`State::X1`] and continue scanning.
    X1Dig,

    /// Accumulate an upper-case hex digit.  Transition to [`State::X1`] and continue scanning.
    X1Uc,

    /// Accumulate a lower-case hex digit.  Transition to [`State::X1`] and continue scanning.
    X1Lc,

    /// Accumulate a decimal digit and complete a numeric escape.
    X2Dig,

    /// Accumulate an upper-case hex digit and complete a numeric escape.
    X2Uc,

    /// Accumulate a lower-case hex digit and complete a numeric escape.
    X2Lc,

    /// Transition to [`State::U`] and continue scanning.
    U,

    /// Transition to [`State::U0`] and continue scanning.
    U0,

    /// Accumulate a decimal digit.  Transition to [`State::UN`] and continue scanning.
    UNDig,

    /// Accumulate an upper-case hex digit.  Transition to [`State::UN`] and continue scanning.
    UNUc,

    /// Accumulate a lower-case hex digit.  Transition to [`State::UN`] and continue scanning.
    UNLc,

    /// Complete a numeric escape.
    UEnd,

    /// Fail due to unrecognized escape sequence.
    EUnrec,

    /// Fail due to invalid escape sequence.
    EInval,

    /// Fail due to unexpected end-of-file.
    EEof,
}

impl Transition {
    /// Gets the transition for the given logical character and sublexer state.
    fn get(kind: Char, state: State) -> Self {
        TRANSITION_MAP[kind as usize + state as usize]
    }

    /// Decodes the transition, returning a tuple consisting of:
    /// - a basis to subtract from the input byte,
    /// - a mask to control numeric accumulation of digits,
    /// - an action to perform, and
    /// - the state to which to transition.
    fn decode(self) -> (u8, u8, Action, State) {
        use Transition::*;
        use Action     as A;
        use State      as S;
        use State::Esc as __;

        match self {
        //  TRAN            BASIS  MASK  ACTION         STATE
            Pass   => (         0,    0, A::Verbatim,   __    ),
            Dig    => ( b'0',         0, A::Simple,     __    ),
            Uc     => ( b'A' - 10,    0, A::Simple,     __    ),
            Lc     => ( b'a' - 10,    0, A::Simple,     __    ),
            X0     => (         0,    0, A::Continue,   S::X0 ),
            X1Dig  => (      b'0', 0xFF, A::Continue,   S::X1 ),
            X1Uc   => (      b'A', 0xFF, A::Continue,   S::X1 ),
            X1Lc   => (      b'a', 0xFF, A::Continue,   S::X1 ),
            X2Dig  => (      b'0', 0xFF, A::Numeric,    __    ),
            X2Uc   => (      b'A', 0xFF, A::Numeric,    __    ),
            X2Lc   => (      b'a', 0xFF, A::Numeric,    __    ),
            U      => (         0,    0, A::Continue,   S::U  ),
            U0     => (         0,    0, A::Continue,   S::U0 ),
            UNDig  => (      b'0', 0xFF, A::Continue,   S::UN ),
            UNUc   => (      b'A', 0xFF, A::Continue,   S::UN ),
            UNLc   => (      b'a', 0xFF, A::Continue,   S::UN ),
            UEnd   => (         0,    0, A::Numeric,    __    ),
            EUnrec => (         0,    0, A::ErrUnknown, __    ),
            EInval => (         0,    0, A::ErrInvalid, __    ),
            EEof   => (         0,    0, A::ErrEof,     __    ),
        }
    }
}

/// Escape sequence sublexer state transition table.
static TRANSITION_MAP: [Transition; Char::COUNT * State::COUNT] = {
    use Transition::*;
    const __: Transition = EInval;
[
  //        Esc     X0      X1      U       U0      UN
  //        \<-     \x<-    \x0<-   \u<-    \u{<-   \u{0<-
  //        ------- ------- ------- ------- ------- -------
  /* 0-9 */ Dig,    X1Dig,  X2Dig,  __,     UNDig,  UNDig,
  /* A-F */ Uc,     X1Uc,   X2Uc,   __,     UNUc,   UNUc,
  /* G-Z */ Uc,     __,     __,     __,     __,     __,
  /* a-f */ Lc,     X1Lc,   X2Lc,   __,     UNLc,   UNLc,
  /* g-z */ Lc,     __,     __,     __,     __,     __,
  /* U u */ U,      __,     __,     __,     __,     __,
  /* X x */ X0,     __,     __,     __,     __,     __,
  /* "'\ */ Pass,   __,     __,     __,     __,     __,
  /* {   */ EUnrec, __,     __,     U0,     __,     __,
  /* }   */ EUnrec, __,     __,     __,     __,     UEnd,
  /* Etc */ EUnrec, __,     __,     __,     __,     __,
  /* Eof */ EEof,   EEof,   EEof,   EEof,   EEof,   EEof,
]};

// ----------------------------------------------------------------------------

/// Simple (single-character) escape sequence table.
static SIMPLE_ESCAPES: [u8; 36] = {
    const __: u8 = b'?';
[
//  0     1     2     3     4     5     6     7     8     9
    0x00, __,   __,   __,   __,   __,   __,   __,   __,   __,

//  Aa    Bb    Cc    Dd    Ee    Ff    Gg    Hh    Ii    Jj
    0x07, 0x08, __,   0x7F, 0x1B, 0x0C, __,   __,   __,   __,

//  Kk    Ll    Mm    Nn    Oo    Pp    Qq    Rr    Ss    Tt
    __,   __,   __,   0x0A, __,   __,   __,   0x0D, 0x20, 0x09,

//  Uu    Vv    Ww    Xx    Yy    Zz
    __,   0x0B, __,   __,   __,   __,
]};

// ----------------------------------------------------------------------------

/// Escape sequence sublexer actions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Action {
    /// Consume the input byte and continue scanning.
    Continue,

    /// Complete a verbatim escape.
    ///
    /// Consume the input byte.  Append the byte verbatim to the string buffer
    /// and return success.
    Verbatim,

    /// Complete a lookup escape.
    ///
    /// Consume the input byte.  If the byte is a valid escape surrogate,
    /// append the corresponding character to the string buffer and return
    /// success.  Otherwise, return failure.
    Simple,

    /// Complete a numeric escape.
    ///
    /// Consume the input byte.  If the numeric accumulator holds a valid
    /// Unicode scalar value, append that character to the string buffer and
    /// return success.  Otherwise, return failure.
    Numeric,

    /// Consume the input byte. Add an 'unknown escape sequence' error and
    /// return failure.
    ErrUnknown,

    /// Add an 'invalid escape sequence' error and return failure.
    ErrInvalid,

    /// Add an 'incomplete escape sequence' error and return failure.
    ErrEof,
}

// ----------------------------------------------------------------------------

impl<I: Iterator<Item = u8>> Lexer<I> {
    /// Attempts to scan an escape sequence.
    pub(super) fn scan_esc(&mut self) -> Result {
        use Action::*;

        let mut state = State::Esc;
        let mut value = 0u32;
        let mut over  = false;

        loop {
            // Read byte as logical character
            let (kind, mut byte) = self.input.classify(&CHARS);

            // Get transition
            let (basis, mask, action, next_state) = Transition::get(kind, state).decode();

            // Convert byte into offset relative to transition's basis
            byte -= basis;

            // Accumulate digit
            let scale  = ((16 ^ 1) & mask ^ 1) as u32; // 16   if digit, 1 if non-digit
            let digit  = (byte     & mask    ) as u32; // byte if digit, 0 if non-digit
            let (v, o) = value.overflowing_mul(scale); value = v; over |= o;
            let (v, o) = value.overflowing_add(digit); value = v; over |= o;

            // Perform transition action
            match action {
                Continue   => (),
                Verbatim   => return self.append_esc_verbatim(byte),
                Simple     => return self.append_esc_simple(byte),
                Numeric    => return self.append_esc_numeric(value, over),
                ErrUnknown => return self.err_esc_unknown(),
                ErrInvalid => return self.err_esc_invalid(),
                ErrEof     => return self.err_esc_incomplete(),
            }

            // Consume input and prepare for next character
            state = next_state;
            self.input.advance();
        }
    }

    fn append_esc_verbatim(&mut self, byte: u8) -> Result {
        self.input.advance();
        self.text.push(byte);
        Ok(())
    }

    fn append_esc_simple(&mut self, byte: u8) -> Result {
        match SIMPLE_ESCAPES[byte as usize] {
            b'?' => self.err_esc_unknown(),
            byte => self.append_esc_verbatim(byte),
        }
    }

    fn append_esc_numeric(&mut self, value: u32, overflow: bool) -> Result {
        self.input.advance();
        if overflow {
            self.err_esc_invalid()
        } else if let Ok(c) = char::try_from(value) {
            let mut bytes = [0; 4];
            self.text.extend(c.encode_utf8(&mut bytes).bytes());
            Ok(())
        } else {
            self.err_esc_invalid()
        }
    }

    fn err_esc_unknown(&mut self) -> Result {
        self.input.advance();
        eprintln!("error: unknown escape sequence");
        Err(())
    }

    fn err_esc_invalid(&mut self) -> Result {
        eprintln!("error: invalid escape sequence");
        Err(())
    }

    fn err_esc_incomplete(&mut self) -> Result {
        eprintln!("error: incomplete escape sequence");
        Err(())
    }
}
