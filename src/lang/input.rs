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

//! Input cursor and logical character set trait.

use std::fmt::Debug;

// ----------------------------------------------------------------------------

/// Trait for logical characters yielded by a [`Cursor`].
///
/// A 'logical character' in ras is effectively a character equivalence class:
/// a value that represents a set of input values which receive identical
/// treatment at some point during lexical analysis.  A 'logical character set'
/// is a set that contains sufficient logical characters to represent all byte
/// values plus an additional logical character to indicate the end of input.
///
pub trait LogicalChar: Copy + Eq {
    /// Logical character that represents a byte beyond the 7-bit ASCII range.
    const NON_ASCII: Self;

    /// Logical character that represents an end-of-input condition.
    const EOF: Self;
}

// ----------------------------------------------------------------------------

/// Input cursor specialized for lexical analysis.
///
/// A `Cursor` takes a sequence of bytes as input and provides a forward-only
/// cursor over a sequence of logical characters.
///
#[derive(Clone, Copy, Debug)]
pub struct Cursor<I: Iterator<Item = u8>> {
    cur:  Option<u8>,
    pos:  usize,
    iter: I,
}

impl<I: Iterator<Item = u8>> Cursor<I> {
    /// Creates a new [`Cursor`] over the given iterator.
    #[inline(always)]
    pub fn new(iter: I) -> Self {
        Self { cur: None, pos: 0, iter }
    }

    /// Advances the cursor to the next byte.
    ///
    /// If the cursor is positioned before the end of input, this method
    /// increments the [`Self::position()`] of the cursor.  Otherwise, this
    /// method does nothing.
    #[inline(always)]
    pub fn advance(&mut self) {
        self.pos += self.cur.is_some() as usize;
        self.cur  = self.iter.next();
    }

    /// Advances the cursor to the next byte if the current byte has the given
    /// value.
    ///
    /// If the cursor is positioned before the end of input and the byte at the
    /// current position equals the given `byte`, this method increments the
    /// [`Self::position()`] of the cursor.  Otherwise, this method does
    /// nothing.
    #[inline(always)]
    pub fn advance_if(&mut self, byte: u8) {
        if self.cur == Some(byte) {
            self.advance();
        }
    }

    /// Returns the current position of the cursor.
    #[inline(always)]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Classifies the byte at the current position of the cursor as some
    /// logical character of type `C` using the given character `map`.
    #[inline(always)]
    pub fn classify<C: LogicalChar>(&self, map: &[C; 128]) -> (C, u8) {
        match self.cur {
            Some(b) if b < 128 => (unsafe { *map.get_unchecked(b as usize) }, b),
            Some(b)            => (C::NON_ASCII,                              b),
            None               => (C::EOF,                                    0),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use Char::*;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Char { Lc, Uc, Etc, Non, Eof }

    impl LogicalChar for Char {
        const NON_ASCII: Self = Non;
        const EOF:       Self = Eof;
    }

    /// Mapping of 7-bit ASCII to logical characters.
    static CHARS: [Char; 128] = {
        const __: Char = Etc;
    [
    //  x0  x1  x2  x3  x4  x5  x6  x7
    //  x8  x9  xA  xB  xC  xD  xE  xF
        __, __, __, __, __, __, __, __, // 0x │········│
        __, __, __, __, __, __, __, __, // 0x │·tn··r··│
        __, __, __, __, __, __, __, __, // 1x │········│
        __, __, __, __, __, __, __, __, // 1x │········│
        __, __, __, __, __, __, __, __, // 2x │ !"#$%&'│
        __, __, __, __, __, __, __, __, // 2x │()*+,-./│
        __, __, __, __, __, __, __, __, // 3x │01234567│
        __, __, __, __, __, __, __, __, // 3x │89:;<=>?│
        __, Uc, Uc, Uc, Uc, Uc, Uc, Uc, // 4x │@ABCDEFG│
        Uc, Uc, Uc, Uc, Uc, Uc, Uc, Uc, // 4x │HIJKLMNO│
        Uc, Uc, Uc, Uc, Uc, Uc, Uc, Uc, // 5x │PQRSTUVW│
        Uc, Uc, Uc, __, __, __, __, __, // 5x │XYZ[\]^_│
        __, Lc, Lc, Lc, Lc, Lc, Lc, Lc, // 6x │`abcdefg│
        Lc, Lc, Lc, Lc, Lc, Lc, Lc, Lc, // 6x │hijklmno│
        Lc, Lc, Lc, Lc, Lc, Lc, Lc, Lc, // 7x │pqrstuvw│
        Lc, Lc, Lc, __, __, __, __, __, // 7x │xyz{|}~░│
    ]};

    #[test]
    fn cursor_use() {
        let mut cursor = super::Cursor::new("Hi!\u{ED}".bytes());

        assert_eq!( cursor.position(),       0          );

        cursor.advance();
        assert_eq!( cursor.classify(&CHARS), (Uc, b'H') );
        assert_eq!( cursor.position(),       0          );

        cursor.advance();
        assert_eq!( cursor.classify(&CHARS), (Lc, b'i') );
        assert_eq!( cursor.position(),       1          );

        cursor.advance();
        assert_eq!( cursor.classify(&CHARS), (Etc, b'!') );
        assert_eq!( cursor.position(),       2           );

        assert_eq!( cursor.classify(&CHARS), (Etc, b'!') );
        assert_eq!( cursor.position(),       2           );

        cursor.advance();
        assert_eq!( cursor.classify(&CHARS), (Non, 0xC3) );
        assert_eq!( cursor.position(),       3           );

        cursor.advance();
        assert_eq!( cursor.classify(&CHARS), (Non, 0xAD) );
        assert_eq!( cursor.position(),       4           );

        cursor.advance();
        assert_eq!( cursor.classify(&CHARS), (Eof, 0)    );
        assert_eq!( cursor.position(),       5           );

        cursor.advance();
        assert_eq!( cursor.classify(&CHARS), (Eof, 0)    );
        assert_eq!( cursor.position(),       5           );
    }
}

/*
Template for logical character set tables:
//  x0      x1      x2      x3      x4      x5      x6      x7
//  x8      x9      xA      xB      xC      xD      xE      xF
    __,     __,     __,     __,     __,     __,     __,     __,     // 0x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 0x │·tnvfr··│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 2x │ !"#$%&'│
    __,     __,     __,     __,     __,     __,     __,     __,     // 2x │()*+,-./│
    __,     __,     __,     __,     __,     __,     __,     __,     // 3x │01234567│
    __,     __,     __,     __,     __,     __,     __,     __,     // 3x │89:;<=>?│
    __,     __,     __,     __,     __,     __,     __,     __,     // 4x │@ABCDEFG│
    __,     __,     __,     __,     __,     __,     __,     __,     // 4x │HIJKLMNO│
    __,     __,     __,     __,     __,     __,     __,     __,     // 5x │PQRSTUVW│
    __,     __,     __,     __,     __,     __,     __,     __,     // 5x │XYZ[\]^_│
    __,     __,     __,     __,     __,     __,     __,     __,     // 6x │`abcdefg│
    __,     __,     __,     __,     __,     __,     __,     __,     // 6x │hijklmno│
    __,     __,     __,     __,     __,     __,     __,     __,     // 7x │pqrstuvw│
    __,     __,     __,     __,     __,     __,     __,     __,     // 7x │xyz{|}~░│
*/
