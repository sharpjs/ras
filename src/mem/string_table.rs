// This file is part of ras, an assembler.
// Copyright (C) 2019 Jeffrey Sharp
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

use std::collections::HashMap;
use std::mem::transmute;

/// An identifier for a string stored in a `StringTable`.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct StringId (u32);

/// A pool of interned strings.
pub struct StringTable {
    // Maps an interned string to its id.  Note that 'static is used here only
    // because Rust has no 'self.  The keys in this map are slices of the
    // `chars` string; their true lifetimes are less than 'static.  Safe
    // lifetimes are enforced via method signatures.
    map: HashMap<&'static str, u32>,

    // Maps an id to the range within `chars` containing the interned string.
    // An id is an index into this vector.  Ranges are represented as start/end
    // positions, because the `Range` type is not copyable.
    table: Vec<(usize, usize)>,

    // Storage for interned strings.  The range `0..mark` is immutable and
    // contains interned strings, concatenated.  The range `mark..chars.len()`
    // is mutable and serves as an accumulator for a pending interned string.
    chars: String,

    // The first mutable position in `chars`.  Also, the count of immutable
    // characters in `chars.  Equal to `chars.len()` unless the caller has a
    // pending interned string.
    mark: usize,
}

impl StringTable {
    const INITIAL_STR_CAPACITY: usize =      1024;
    const INITIAL_CHR_CAPACITY: usize = 16 * 1024;

    /// Creates a new `StringTable`.  Initially, the table contains only the
    /// empty string mapped to the default [`StringId`].
    pub fn new() -> Self {
        let mut table = Self {
            map:   HashMap::with_capacity(Self::INITIAL_STR_CAPACITY),
            table: Vec    ::with_capacity(Self::INITIAL_STR_CAPACITY),
            chars: String ::with_capacity(Self::INITIAL_CHR_CAPACITY),
            mark:  0,
        };
        table.intern_accum(); // StringId(0) => ""
        table
    }

    /// Returns the number of interned strings.
    #[inline]
    pub fn interned_count(&self) -> usize {
        self.table.len()
    }

    /// Returns the number of characters used for interned string storage,
    /// including any padding.
    #[inline]
    pub fn interned_len(&self) -> usize {
        self.mark
    }

    /// Returns the number of characters in the pending-string accumulator.
    #[inline]
    pub fn pending_len(&self) -> usize {
        self.chars.len() - self.mark
    }

    /// Returns the contents of the pending-string accumulator.
    #[inline]
    pub fn pending(&self) -> &str {
        &self.chars[self.mark..]
    }

    /// Appends the given character `c` to the pending-string accumulator.
    #[inline]
    pub fn push(&mut self, c: char) {
        self.chars.push(c);
    }

    /// Appends the given string slice `s` to the pending-string accumulator.
    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.chars.push_str(s);
    }

    /// Clears the pending-string accumulator.
    #[inline]
    pub fn clear_pending(&mut self) {
        self.chars.truncate(self.mark);
    }

    /// Interns the contents of and clears the pending-string accumulator.
    /// Returns a [`StringId`] that uniquely identifies the interned string.
    pub fn intern(&mut self) -> StringId {
        // String is present in accumulator

        // Check if string is interned already
        if let Some(&id) = self.map.get(self.pending()) {
            self.clear_pending();
            return StringId(id)
        }

        // Intern the string
        self.intern_accum()
    }

    /// Interns the given string `s` and clears the pending-string accumulator.
    /// Returns a [`StringId`] that uniquely identifies the interned string.
    pub fn intern_str(&mut self, s: &str) -> StringId {
        // Reset accumulator
        self.clear_pending();

        // Check if string is interned already
        if let Some(&id) = self.map.get(s) {
            return StringId(id)
        }

        // Intern a copy of the string
        self.push_str(s);
        self.intern_accum()
    }

    fn intern_accum(&mut self) -> StringId {
        let start = self.mark;
        let end   = self.chars.len();

        // Re-reference s from interned storage
        let s = &self.chars[start..end];
        let s = unsafe { transmute::<&'_ str, &'static str>(s) };

        // Add entry to table
        let index = self.table.len() as u32;
                    self.table.push((start, end));

        // Add entry to map
        self.map.insert(s, index);

        // Mark range as immutable
        self.mark = end;

        // Return table index as id
        StringId(index)
    }

    /// Retuns a reference to the interned string with the given `id`.
    pub fn get(&self, id: StringId) -> &str {
        let index        = id.0 as usize;
        let (start, end) = self.table[index];
        &self.chars[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Prefixes guarantee that rustc will emit two separate "Hello" strings.
    const A: &str = "A Hello";
    const B: &str = "B Hello";
    const C: &str = "C olleH";

    #[test]
    fn default_id() {
        let table = StringTable::new();

        assert_eq!("", table.get(StringId::default()));
    }

    #[test]
    fn intern() {
        let mut table = StringTable::new();

        table.push('H');
        table.push('e');
        table.push_str("llo");
        let a_id = table.intern();

        table.push('H');
        table.push('e');
        table.push_str("llo");
        let b_id = table.intern();

        table.push_str("oll");
        table.push('e');
        table.push('H');
        let c_id = table.intern();

        assert_eq!(a_id, b_id);
        assert_ne!(a_id, c_id);
        assert_eq!("Hello", table.get(b_id) );
        assert_eq!("olleH", table.get(c_id) );
    }

    #[test]
    fn intern_str() {
        let a_str = &A[2..];
        let b_str = &B[2..];
        let c_str = &C[2..];

        assert_ne!(a_str.as_ptr(), b_str.as_ptr());

        let mut table = StringTable::new();
        let a_id = table.intern_str(a_str);
        let b_id = table.intern_str(b_str);
        let c_id = table.intern_str(c_str);

        assert_eq!(a_id, b_id);
        assert_ne!(a_id, c_id);
        assert_eq!("Hello", table.get(b_id) );
        assert_eq!("olleH", table.get(c_id) );
    }
}

