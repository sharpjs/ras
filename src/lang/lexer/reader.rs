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

/// Trait for types that have a compile-time constant default value.
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
