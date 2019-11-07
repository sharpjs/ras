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

// ----------------------------------------------------------------------------

/// Trait for logical character types used with [`Reader`].
pub trait LogChar: Copy {

    /// Logical character representing a byte beyond the 7-bit ASCII range.
    const EXT: Self;

    /// Logical character representing an end-of-file condition.
    const EOF: Self;
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
    /// `(C::EOF, 0)`, and the reader's position remains unchanged.
    #[inline(always)]
    pub fn next<C>(&mut self, map: &[C; 128]) -> (C, u8) where C: LogChar {
        let p = self.ptr;
        if p == self.end {
            return (C::EOF, 0)
        }
        let byte = unsafe { *p } as i8;
        self.ptr = unsafe { p.offset(1) };
        (
            if byte >= 0 {
                map[byte as usize]
            } else {
                C::EXT
            },
            byte as u8
        )
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
    enum Char { Lc, Uc, Etc, Ext, Eof }

    impl LogChar for Char {
        const EXT: Self = Ext;
        const EOF: Self = Eof;
    }

    /// Mapping of 7-bit ASCII bytes to `Char` logical characters.
    static CHARS: [Char; 128] = {
        const __: Char = Etc;
    [
    //  [x0] [x1] [x2] [x3] [x4] [x5] [x6] [x7]
    //  [x8] [x9] [xA] [xB] [xC] [xD] [xE] [xF]
        __,  __,  __,  __,  __,  __,  __,  __,  // [0x] ........
        __,  __,  __,  __,  __,  __,  __,  __,  // [0x] .tn..r..
        __,  __,  __,  __,  __,  __,  __,  __,  // [1x] ........
        __,  __,  __,  __,  __,  __,  __,  __,  // [1x] ........
        __,  __,  __,  __,  __,  __,  __,  __,  // [2x]  !"#$%&'
        __,  __,  __,  __,  __,  __,  __,  __,  // [2x] ()*+,-./
        __,  __,  __,  __,  __,  __,  __,  __,  // [3x] 01234567
        __,  __,  __,  __,  __,  __,  __,  __,  // [3x] 89:;<=>?
        __,  Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  // [4x] @ABCDEFG
        Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  // [4x] HIJKLMNO
        Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  Uc,  // [5x] PQRSTUVW
        Uc,  Uc,  Uc,  __,  __,  __,  __,  __,  // [5x] XYZ[\]^_
        __,  Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  // [6x] `abcdefg
        Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  // [6x] hijklmno
        Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  Lc,  // [7x] pqrstuvw
        Lc,  Lc,  Lc,  __,  __,  __,  __,  __,  // [7x] xyz{|}~. <- DEL
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
        let mut reader = Reader::new(b"Hi!\xED");

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

        assert_eq!( reader.next(&CHARS), (Ext, b'\xED') );
        assert_eq!( reader.position(),   4           );

        assert_eq!( reader.next(&CHARS), (Eof, 0)    );
        assert_eq!( reader.position(),   4           );
    }

    #[test]
    fn reader_preceding()
    {
        let mut reader = Reader::new(b"ab");

        assert_eq!( reader.preceding(0),   b"" );

        let _ = reader.next(&CHARS);

        assert_eq!( reader.preceding(0),   b"" );
        assert_eq!( reader.preceding(1),  b"a" );

        let _ = reader.next(&CHARS);

        assert_eq!( reader.preceding(0),   b"" );
        assert_eq!( reader.preceding(1),  b"b" );
        assert_eq!( reader.preceding(2), b"ab" );
    }

    #[test]
    #[should_panic]
    fn reader_preceding_panic()
    {
        let reader = Reader::new(b"ab");

        let _ = reader.preceding(1);
    }

    #[test]
    fn reader_remaining()
    {
        let mut reader = Reader::new(b"ab");

        assert_eq!( reader.remaining(), b"ab" );

        let _ = reader.next(&CHARS);

        assert_eq!( reader.remaining(), b"b" );

        let _ = reader.next(&CHARS);

        assert_eq!( reader.remaining(), b"" );

        let _ = reader.next(&CHARS);

        assert_eq!( reader.remaining(), b"" );
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
