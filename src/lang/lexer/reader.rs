//! Character Reader
//
// This file is part of ras, an assembler.
// Copyright 2020 Jeffrey Sharp
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
use std::ops::Range;
use std::slice;

// ----------------------------------------------------------------------------

/// Trait for logical characters yielded by a [`Reader`].
///
/// A 'logical character' in ras is effectively a character equivalence class:
/// a value that represents a set of byte values which receive identical
/// treatment during lexical analysis.  A 'logical character set' is a set that
/// contains sufficient logical characters to represent all byte values.
///
pub trait LogicalChar: Copy + Eq {

    /// Logical character representing a byte beyond the 7-bit ASCII range.
    const NON_ASCII: Self;

    /// Logical character representing an end-of-file condition.
    const EOF: Self;
}

// ----------------------------------------------------------------------------

/// Input reader specialized for lexical analysis.
///
/// A `Reader` takes a slice of bytes as input and provides a simple rewindable
/// cursor over a sequence of logical characters.
///
#[derive(Clone, Copy)]
pub struct Reader<'a> {
    ptr: *const u8, // pointer to current position
    old: *const u8, // pointer to previous position
    beg: *const u8, // pointer to first byte of slice
    end: *const u8, // pointer to first byte after slice
    _lt: PhantomData<&'a ()>,
}

impl<'a> Reader<'a> {
    // Safety: Similar to std::slice::Iter.  Performs pointer arithmetic and
    // dereferences pointers to bytes within a slice of bytes.  Safety is
    // ensured by checks against the slice bounds.

    /// Creates a new [`Reader`] over the given slice of bytes.
    ///
    /// # Panics
    ///
    /// Panics if the length of `bytes` is greater than `isize::MAX`.
    ///
    pub fn new(bytes: &'a [u8]) -> Self {
        if bytes.len() > isize::MAX as usize {
            // Rust pointer arithmetic requires offsets <= isize::MAX
            panic!("Input exceeds maximum supported size of {} bytes.", isize::MAX)
        }

        let Range { start: beg, end } = bytes.as_ptr_range();

        Self { ptr: beg, old: beg, beg, end, _lt: PhantomData }
    }

    /// Returns the position of the next byte to be read.
    ///
    /// The position is equivalent to the count of bytes that have been read.
    ///
    #[inline(always)]
    pub fn position(&self) -> usize {
        self.ptr as usize - self.beg as usize
    }

    /// Reads the next byte, advances the reader, and returns both the byte and
    /// its corresponding logical character from the given character set `map`.
    ///
    /// If the reader is positioned at the end of input, this method returns
    /// `(C::EOF, 0)`, and the reader's position remains unchanged.
    pub fn read<C>(&mut self, map: &[C; 128]) -> (C, u8) where C: LogicalChar {
        // Update rewind position to current
        let p = self.ptr;
        self.old = p;

        // Detect EOF
        if p == self.end {
            return (C::EOF, 0)
        }

        // Read byte and advance
        let byte = unsafe { *p };
        self.ptr = unsafe { p.add(1) };

        // Map byte to logical character
        let c = if byte as i8 >= 0 {
            unsafe { *map.get_unchecked(byte as usize) }
        } else {
            C::NON_ASCII // beyond 7-bit ASCII range
        };

        (c, byte)
    }

    /// Unreads the most-recently-read logical character.
    ///
    /// This method can have effect only once after each call to [`read`].  It
    /// is safe to call this method an arbitrary number of times before or
    /// after [`read`]; these additional calls have no effect and leave the
    /// reader state unchanged.
    ///
    #[inline(always)]
    pub fn unread(&mut self) {
        self.ptr = self.old;
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
    enum Char { Lc, Uc, Etc, Non, Eof }

    impl LogicalChar for Char {
        const NON_ASCII: Self = Non;
        const EOF:       Self = Eof;
    }

    /// Mapping of 7-bit ASCII to logical characters.
    static CHARS: [Char; 128] = {
        const __: Char = Etc;
    [
    //  xx0 xx1 xx2 xx3 xx4 xx5 xx6 xx7
        __, __, __, __, __, __, __, __, // 00x │········│
        __, __, __, __, __, __, __, __, // 01x │·tn··r··│
        __, __, __, __, __, __, __, __, // 02x │········│
        __, __, __, __, __, __, __, __, // 03x │········│
        __, __, __, __, __, __, __, __, // 04x │ !"#$%&'│
        __, __, __, __, __, __, __, __, // 05x │()*+,-./│
        __, __, __, __, __, __, __, __, // 06x │01234567│
        __, __, __, __, __, __, __, __, // 07x │89:;<=>?│
        __, Uc, Uc, Uc, Uc, Uc, Uc, Uc, // 10x │@ABCDEFG│
        Uc, Uc, Uc, Uc, Uc, Uc, Uc, Uc, // 11x │HIJKLMNO│
        Uc, Uc, Uc, Uc, Uc, Uc, Uc, Uc, // 12x │PQRSTUVW│
        Uc, Uc, Uc, __, __, __, __, __, // 13x │XYZ[\]^_│
        __, Lc, Lc, Lc, Lc, Lc, Lc, Lc, // 14x │`abcdefg│
        Lc, Lc, Lc, Lc, Lc, Lc, Lc, Lc, // 15x │hijklmno│
        Lc, Lc, Lc, Lc, Lc, Lc, Lc, Lc, // 16x │pqrstuvw│
        Lc, Lc, Lc, __, __, __, __, __, // 17x │xyz{|}~·│
    ]};

    #[test]
    fn reader_empty() {
        let mut reader = Reader::new(b"");

        assert_eq!( reader.position(),   0        );

        reader.unread();
        assert_eq!( reader.position(),   0        );

        assert_eq!( reader.read(&CHARS), (Eof, 0) );
        assert_eq!( reader.position(),   0        );

        reader.unread();
        assert_eq!( reader.position(),   0        );
    }

    #[test]
    fn reader_read() {
        let mut reader = Reader::new(b"Hi!\xED");

        assert_eq!( reader.position(),   0              );

        reader.unread();
        assert_eq!( reader.position(),   0              );

        assert_eq!( reader.read(&CHARS), (Uc,  b'H')    );
        assert_eq!( reader.position(),   1              );

        assert_eq!( reader.read(&CHARS), (Lc,  b'i')    );
        assert_eq!( reader.position(),   2              );

        assert_eq!( reader.read(&CHARS), (Etc, b'!')    );
        assert_eq!( reader.position(),   3              );

        reader.unread();
        assert_eq!( reader.position(),   2              );

        reader.unread();
        assert_eq!( reader.position(),   2              );

        assert_eq!( reader.read(&CHARS), (Etc, b'!')    );
        assert_eq!( reader.position(),   3              );

        assert_eq!( reader.read(&CHARS), (Non, b'\xED') );
        assert_eq!( reader.position(),   4              );

        assert_eq!( reader.read(&CHARS), (Eof, 0)       );
        assert_eq!( reader.position(),   4              );

        reader.unread();
        assert_eq!( reader.position(),   4              );
    }

    #[test]
    fn reader_preceding() {
        let mut reader = Reader::new(b"ab");

        assert_eq!( reader.preceding(0),   b"" );

        let _ = reader.read(&CHARS);

        assert_eq!( reader.preceding(0),   b"" );
        assert_eq!( reader.preceding(1),  b"a" );

        let _ = reader.read(&CHARS);

        assert_eq!( reader.preceding(0),   b"" );
        assert_eq!( reader.preceding(1),  b"b" );
        assert_eq!( reader.preceding(2), b"ab" );

        let _ = reader.read(&CHARS);

        assert_eq!( reader.preceding(0),   b"" );
        assert_eq!( reader.preceding(1),  b"b" );
        assert_eq!( reader.preceding(2), b"ab" );
    }

    #[test]
    #[should_panic]
    fn reader_preceding_panic() {
        let reader = Reader::new(b"ab");

        let _ = reader.preceding(1);
    }

    #[test]
    fn reader_remaining() {
        let mut reader = Reader::new(b"ab");

        assert_eq!( reader.remaining(), b"ab" );

        let _ = reader.read(&CHARS);

        assert_eq!( reader.remaining(), b"b" );

        let _ = reader.read(&CHARS);

        assert_eq!( reader.remaining(), b"" );

        let _ = reader.read(&CHARS);

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

/*
Template for logical character set tables:

//  x0      x1      x2      x3      x4      x5      x6      x7
//  x8      x9      xA      xB      xC      xD      xE      xF
    __,     __,     __,     __,     __,     __,     __,     __,     // 0x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 0x │·tn··r··│
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
    __,     __,     __,     __,     __,     __,     __,     __,     // 7x │xyz{|}~·│
*/
