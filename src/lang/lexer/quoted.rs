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

//! Quoted literal sublexer.

use crate::lang::input::LogicalChar;
use super::*;

// ----------------------------------------------------------------------------

/// Logical characters recognized by the quoted literal sublexer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Char {
    //          ╭───────── is_lf
    //          │ ╭─────── is_cr
    //          │ │ ╭┬──── action
    //          │ │ ││              line +=   inc =
    Text   = 0b_0_0_00, // other    inc       0
    BSlash = 0b_0_0_10, // \        inc       0
    Eof    = 0b_0_0_11, // EOF      inc       0
    Cr     = 0b_0_1_00, // \r       inc       1
    Lf     = 0b_1_0_00, // \n       1         0
}

impl Char {
    /// Returns `1` if the character is a carriage return, `0` otherwise.
    fn is_cr(self) -> u8 {
        self as u8 >> 2
    }

    /// Returns `1` if the character is a line feed, `0` otherwise.
    fn is_lf(self) -> u8 {
        self as u8 >> 3
    }

    /// Given whether the character is the terminating quote, returns the action to perform.
    fn action(self, is_end: bool) -> Action {
        // SAFETY: All values are within range of `Action`.
        unsafe {
            std::mem::transmute(
                self   as u8 & 3 |  // 0-3
                is_end as u8        // 0 or 1
            )
        }
    }
}

impl LogicalChar for Char {
    const NON_ASCII: Self = Self::Text;
    const EOF:       Self = Self::Eof;
}

/// Mapping of 7-bit ASCII to logical characters.
static CHARS: [Char; 128] = {
    use Char::*;
    const __: Char = Text;
[
//  x0      x1      x2      x3      x4      x5      x6      x7
//  x8      x9      xA      xB      xC      xD      xE      xF
    __,     __,     __,     __,     __,     __,     __,     __,     // 0x │········│
    __,     __,     Lf,     __,     __,     Cr,     __,     __,     // 0x │·tnvfr··│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 2x │ !"#$%&'│
    __,     __,     __,     __,     __,     __,     __,     __,     // 2x │()*+,-./│
    __,     __,     __,     __,     __,     __,     __,     __,     // 3x │01234567│
    __,     __,     __,     __,     __,     __,     __,     __,     // 3x │89:;<=>?│
    __,     __,     __,     __,     __,     __,     __,     __,     // 4x │@ABCDEFG│
    __,     __,     __,     __,     __,     __,     __,     __,     // 4x │HIJKLMNO│
    __,     __,     __,     __,     __,     __,     __,     __,     // 5x │PQRSTUVW│
    __,     __,     __,     __,     BSlash, __,     __,     __,     // 5x │XYZ[\]^_│
    __,     __,     __,     __,     __,     __,     __,     __,     // 6x │`abcdefg│
    __,     __,     __,     __,     __,     __,     __,     __,     // 6x │hijklmno│
    __,     __,     __,     __,     __,     __,     __,     __,     // 7x │pqrstuvw│
    __,     __,     __,     __,     __,     __,     __,     __,     // 7x │xyz{|}~░│
]};

// ----------------------------------------------------------------------------

/// Actions performed by the quoted literal sublexer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Action {
    /// Consume the input byte.  Append the byte to the literal and continue scanning.
    Append,

    /// Consume the input byte and yield a quoted literal token.
    Accept,

    /// Consume the input byte.  Scan an escape sequence, then continue scanning.
    Escape,

    /// Add an 'unterminated quoted literal' error and fail.
    Unterm,
}

// ----------------------------------------------------------------------------

impl<I: Iterator<Item = u8>> Lexer<I> {
    /// Attempts to scan a string literal.
    #[inline]
    pub(super) fn scan_str(&mut self) -> Option<Token> {
        self.scan_quoted(b'"', Token::Str)
    }

    /// Attempts to scan a character literal.
    #[inline]
    pub(super) fn scan_char(&mut self) -> Option<Token> {
        let token = self.scan_quoted(b'\'', Token::Char)?;

        let mut chars = self.str().chars();

        match chars.next() {
            None => {
                eprintln!("error: empty character literal");
                None
            },
            Some(c) if chars.next().is_none() => {
                self.char = c;
                Some(token)
            },
            _ => {
                eprintln!("error: character literal contains more than one character");
                None
            },
        }
    }

    /// Attempts to scan a the given quoted literal `token` with the given `end_byte`.
    fn scan_quoted(&mut self, end_byte: u8, token: Token) -> Option<Token> {
        use Action::*;

        // Consume the beginning quote
        self.input.advance();

        // Prepare the content buffer
        self.str_buf.clear();

        // Initialize loop state
        let mut line     = self.line;
        let mut after_cr = 0;

        let result = loop {
            // Read logical character
            let (kind, byte) = self.input.classify(&CHARS);

            // Get action
            let action = kind.action(byte == end_byte);

            // Perform action
            match action {
                Append => {
                    // Consume the read byte
                    self.input.advance();

                    // Append the byte verbatim
                    self.str_buf.push(byte);

                    // Accumulate line number
                    line     += kind.is_lf() as usize | after_cr;
                    after_cr  = kind.is_cr() as usize;
                },
                Accept => {
                    // Consume the ending quote
                    self.input.advance();

                    // Succeed
                    break Some(token)
                },
                Escape => {
                    // Consume the backslash
                    self.input.advance();

                    // Attempt to scan the rest of the escape sequence
                    let _ = self.scan_esc();
                },
                Unterm => {
                    // TODO: Add unterminated string error
                    eprintln!("error: unterminated string or character literal (TODO)");

                    // Fail
                    break None
                },
            }
        };

        // Store lexer state
        self.line      = line;
        self.line_next = line;

        result
    }
}
