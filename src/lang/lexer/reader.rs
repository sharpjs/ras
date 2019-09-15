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

use std::fmt::{Debug, Formatter, Result};
use std::marker::PhantomData;
use std::slice;

/// Trait for types whose instances form a logical character set.
pub trait CharSet: Copy {
    /// The default value of the type.
    const DEFAULT: Self;
}

// ----------------------------------------------------------------------------

/// Input reader specialized for lexical analysis.  A `Reader` takes a slice of
/// bytes as input and provides a simple rewindable cursor over a sequence of
/// logical characters (effectively, character equivalence classes).
///
#[derive(Clone, Copy)]
pub struct Reader<'a> {
    ptr: *const u8,
    beg: *const u8,
    end: *const u8,
    _lt: PhantomData<&'a ()>,
}

impl<'a> Reader<'a> {
    // Safety: Similar to std::slice::Iter.  Performs pointer arithmetic and
    // dereferences pointers to bytes within a slice of bytes.  Safety is
    // ensured by checks against the slice bounds.

    /// Creates a new [`Reader`] over the given slice of bytes.
    #[inline(always)]
    pub fn new(bytes: &'a [u8]) -> Self {
        let beg = bytes.as_ptr();
        let end = unsafe { beg.add(bytes.len()) };

        Self { ptr: beg, beg, end, _lt: PhantomData }
    }

    /// Returns the position of the next byte to be read.
    #[inline(always)]
    pub fn position(&self) -> usize {
        self.ptr as usize - self.beg as usize
    }

    /// Reads the next byte, advances the reader, and returns both the byte and
    /// its corresponding logical character from the given character set `map`.
    ///
    /// If the reader is positioned at the end of input, this method returns
    /// `(C::DEFAULT, 0)`, and the reader's position remains unchanged.
    #[inline(always)]
    pub fn next<C>(&mut self, map: &[C; 256]) -> (C, u8) where C: CharSet {
        let p = self.ptr;
        if p == self.end {
            (C::DEFAULT, 0)
        } else {
            unsafe {
                self.ptr = p.offset(1);
                let byte = *p;
                (map[byte as usize], byte)
            }
        }
    }

    /// Rewinds the reader by one byte.
    ///
    /// # Panics
    ///
    /// Panics if the reader is positioned at the beginning of input.
    ///
    #[inline(always)]
    pub fn rewind(&mut self) {
        let p = self.ptr;
        if p == self.beg {
            panic!("Attempted to rewind past the beginning of input.")
        }
        self.ptr = unsafe { p.offset(-1) };
    }

    /// Returns a slice of the `len` bytes preceding the next byte to be read.
    ///
    /// # Panics
    ///
    /// Panics if `len` exceeds the count of bytes that have been read.
    ///
    #[inline(always)]
    pub fn preceding(&self, len: usize) -> &'a [u8] {
        if len > self.position() {
            panic!("Attempted to obtain a slice before the beginning of input.")
        }
        unsafe {
            slice::from_raw_parts(self.ptr.sub(len), len)
        }
    }

    /// Returns a slice of the bytes remaining to be read.
    #[inline(always)]
    pub fn remaining(&self) -> &'a [u8] {
        let len = self.end as usize - self.ptr as usize;
        unsafe {
            slice::from_raw_parts(self.ptr, len)
        }
    }
}

impl<'a> Debug for Reader<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Reader {:X?}", self.remaining())
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use Char::*;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Char { Lc, Uc, Etc, Eof }

    impl CharSet for Char {
        const DEFAULT: Self = Self::Eof;
    }

    /// Mapping of bytes to `Char` logical characters.
    static CHARS: [Char; 256] = {
        const __: Char = Etc;
    [
    //  7-bit ASCII characters
    //  x0   x1   x2   x3   x4   x5   x6   x7   CHARS
        __,  __,  __,  __,  __,  __,  __,  __,  // ........
        __,  __,  __,  __,  __,  __,  __,  __,  // .tn..r..
        __,  __,  __,  __,  __,  __,  __,  __,  // ........
        __,  __,  __,  __,  __,  __,  __,  __,  // ........
        __,  __,  __,  __,  __,  __,  __,  __,  //  !"#$%&'
        __,  __,  __,  __,  __,  __,  __,  __,  // ()*+,-./
        __,  __,  __,  __,  __,  __,  __,  __,  // 01234567
        __,  __,  __,  __,  __,  __,  __,  __,  // 89:;<=>?
        __,  Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  // @ABCDEFG
        Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  // HIJKLMNO
        Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  // PQRSTUVW
        Uc,  Uc,  Uc,  __,  __,  __,  __,  __,  // XYZ[\]^_
        __,  Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  // `abcdefg
        Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  // hijklmno
        Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  // pqrstuvw
        Lc,  Lc,  Lc,  __,  __,  __,  __,  __,  // xyz{|}~. <- DEL

    //  UTF-8 multibyte sequences
    //  x8   x9   xA   xB   xC   xD   xE   xF   RANGE
        __,  __,  __,  __,  __,  __,  __,  __,  // 80-87
        __,  __,  __,  __,  __,  __,  __,  __,  // 88-8F
        __,  __,  __,  __,  __,  __,  __,  __,  // 90-97
        __,  __,  __,  __,  __,  __,  __,  __,  // 98-9F
        __,  __,  __,  __,  __,  __,  __,  __,  // A0-A7
        __,  __,  __,  __,  __,  __,  __,  __,  // A8-AF
        __,  __,  __,  __,  __,  __,  __,  __,  // B0-B7
        __,  __,  __,  __,  __,  __,  __,  __,  // B8-BF
        __,  __,  __,  __,  __,  __,  __,  __,  // C0-C7
        __,  __,  __,  __,  __,  __,  __,  __,  // C8-CF
        __,  __,  __,  __,  __,  __,  __,  __,  // D0-D7
        __,  __,  __,  __,  __,  __,  __,  __,  // D8-DF
        __,  __,  __,  __,  __,  __,  __,  __,  // E0-E7
        __,  __,  __,  __,  __,  __,  __,  __,  // E8-EF
        __,  __,  __,  __,  __,  __,  __,  __,  // F0-F7
        __,  __,  __,  __,  __,  __,  __,  __,  // F8-FF
    ]};

    #[test]
    fn reader_empty() {
        let mut reader = Reader::new(b"");

        assert_eq!( reader.position(),   0        );

        assert_eq!( reader.next(&CHARS), (Eof, 0) );
        assert_eq!( reader.position(),   0        );
    }

    #[test]
    fn reader_next() {
        let mut reader = Reader::new(b"Hi!");

        assert_eq!( reader.position(),   0           );

        assert_eq!( reader.next(&CHARS), (Uc,  b'H') );
        assert_eq!( reader.position(),   1           );

        assert_eq!( reader.next(&CHARS), (Lc,  b'i') );
        assert_eq!( reader.position(),   2           );

        assert_eq!( reader.next(&CHARS), (Etc, b'!') );
        assert_eq!( reader.position(),   3           );

        reader.rewind();
        assert_eq!( reader.position(),   2           );

        assert_eq!( reader.next(&CHARS), (Etc, b'!') );
        assert_eq!( reader.position(),   3           );

        assert_eq!( reader.next(&CHARS), (Eof, 0)    );
        assert_eq!( reader.position(),   3           );
    }

    #[test]
    fn reader_debug_empty() {
        let reader = Reader::new(b"");

        assert_eq!( format!("{:?}", reader), "Reader []" );
    }

    #[test]
    fn reader_debug_not_empty() {
        let reader = Reader::new(b"X+1");

        assert_eq!( format!("{:?}", reader), "Reader [58, 2B, 31]" );
    }
}
