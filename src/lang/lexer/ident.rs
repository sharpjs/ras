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

use crate::lang::input::LogicalChar;
use super::*;

///! Identifier sublexer.

// ----------------------------------------------------------------------------

/// Logical characters recognized by the main lexer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
enum Char {
    Ident,
    Other
}

impl Char {
    /// Count of logical characters.
    const COUNT: usize = Self::Other as usize; // / State::COUNT + 1;
}

impl LogicalChar for Char {
    const NON_ASCII: Self = Self::Ident;
    const EOF:       Self = Self::Other;
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
    __,     __,     __,     __,     __,     __,     __,     __,     // 2x │ !"#$%&'│
    __,     __,     __,     __,     __,     __,     Ident,  __,     // 2x │()*+,-./│
    Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 3x │01234567│
    Ident,  Ident,  __,     __,     __,     __,     __,     __,     // 3x │89:;<=>?│
    __,     Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 4x │@ABCDEFG│
    Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 4x │HIJKLMNO│
    Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 5x │PQRSTUVW│
    Ident,  Ident,  Ident,  __,     __,     __,     __,     __,     // 5x │XYZ[\]^_│
    __,     Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 6x │`abcdefg│
    Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 6x │hijklmno│
    Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 7x │pqrstuvw│
    Ident,  Ident,  Ident,  __,     __,     __,     __,     __,     // 7x │xyz{|}~░│
]};

// ----------------------------------------------------------------------------

impl<I: Iterator<Item = u8>> Lexer<I> {
    /// Scans an identifier or macro parameter.
    pub(super) fn scan_ident(&mut self, variant: u8) -> Token {
        use Char::*;

        let input = &mut self.input;
        let buf   = &mut self.str_buf;

        buf.clear();

        loop {
            let (kind, byte) = input.classify(&CHARS);

            match kind {
                Ident => { buf.push(byte); input.advance(); },
                Other => break,
            }
        }

        Token::Ident.variant(variant)
    }
}
