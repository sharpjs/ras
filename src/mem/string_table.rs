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
pub struct StringId (usize);

/// A table of interned strings, each identified by a [`StringId`].
pub struct StringTable {
    // Maps an interned string to its id.  Note that 'static is used here only
    // because Rust has no 'self.  The keys in this map are slices of the
    // `chars` string; their true lifetimes are less than 'static.  Safe
    // lifetimes are enforced via method signatures.
    map: HashMap<&'static str, usize>,

    // Maps an id to the range within `chars` containing the interned string.
    // An id is an index into this vector.  Ranges are represented as start/end
    // positions, because the `Range` type is not copyable.
    table: Vec<(usize, usize)>,

    // Storage for interned strings.  The range `0..mark` is immutable and
    // contains interned strings, concatenated.  The range `mark..` is mutable
    // and serves as an accumulator for a pending string.
    chars: String,

    // The first mutable position in `chars`, at which the table can accumulate
    // a pending string for later interning.  Also, the count of immutable
    // characters in `chars`.  Equal to `chars.len()` unless a non-empty string
    // is pending.
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
        table.intern(); // StringId(0) => ""
        table
    }

    /// Returns the number of interned strings.
    #[inline]
    pub fn count(&self) -> usize {
        self.table.len()
    }

    /// Returns the size of interned string storage, in bytes.
    #[inline]
    pub fn size(&self) -> usize {
        self.mark
    }

    /// Borrows the pending string.
    #[inline]
    pub fn pending(&self) -> &str {
        &self.chars[self.mark..]
    }

    /// Appends the given character to the pending string.
    #[inline]
    pub fn push_pending(&mut self, c: char) {
        self.chars.push(c);
    }

    /// Appends the given string slice to the pending string.
    #[inline]
    pub fn push_pending_str(&mut self, s: &str) {
        self.chars.push_str(s);
    }

    /// Resets the pending string to empty.
    #[inline]
    pub fn clear_pending(&mut self) {
        self.chars.truncate(self.mark);
    }

    /// Interns the the pending string.  Returns a [`StringId`] that uniquely
    /// identifies the interned string.  Resets the pending string area to
    /// empty.
    pub fn intern_pending(&mut self) -> StringId {
        // Check if pending string is interned already
        if let Some(&id) = self.map.get(self.pending()) {
            self.clear_pending();
            return StringId(id)
        }

        // Intern the pending string
        self.intern()
    }

    /// Interns the given string.  Returns a [`StringId`] that uniquely
    /// identifies the interned string.  Resets the pending string area to
    /// empty.
    pub fn intern_str(&mut self, s: &str) -> StringId {
        self.clear_pending();

        // Check if given string is interned already
        if let Some(&id) = self.map.get(s) {
            return StringId(id)
        }

        // Intern a copy of the given string
        self.push_pending_str(s);
        self.intern()
    }

    // Interns the pending string, which is known to not be interned already.
    fn intern(&mut self) -> StringId {
        // Get bounds
        let chars = &self.chars;
        let start =  self.mark;
        let end   = chars.len();

        // Borrow the string from interned storage.
        // SAFETY: 'static because Rust has no 'self.  See comments in struct.
        // Safe lifetimes are enforced via method signatures.
        let string = &chars[start..end];
        let string = unsafe { transmute::<&'_ str, &'static str>(string) };

        // Add entry to table
        let table = &mut self.table;
        let index = table.len();
        table.push((start, end));

        // Add entry to map
        self.map.insert(string, index);

        // Mark range as immutable
        self.mark = end;

        // Return table index as id
        StringId(index)
    }

    /// Borrows the interned string with the given `id`.
    pub fn get(&self, id: StringId) -> &str {
        let (start, end) = self.table[id.0 as usize];
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
    fn initial() {
        let table = StringTable::new();

        assert_eq!(1,  table.count());
        assert_eq!(0,  table.size());
        assert_eq!("", table.pending());
        assert_eq!("", table.get(StringId::default()));
    }

    #[test]
    fn push_pending() {
        let mut table = StringTable::new();

        table.push_pending('H');
        table.push_pending('e');

        assert_eq!(1,    table.count());
        assert_eq!(0,    table.size());
        assert_eq!("He", table.pending());
        assert_eq!("",   table.get(StringId::default()));
    }

    #[test]
    fn push_pending_str() {
        let mut table = StringTable::new();

        table.push_pending_str("He");
        table.push_pending_str("llo");

        assert_eq!(1,       table.count());
        assert_eq!(0,       table.size());
        assert_eq!("Hello", table.pending());
        assert_eq!("",      table.get(StringId::default()));
    }

    #[test]
    fn intern_pending() {
        let mut table = StringTable::new();

        table.push_pending('H');
        table.push_pending('e');
        table.push_pending_str("llo");
        let a_id = table.intern_pending();

        assert_eq!(StringId(1), a_id);
        assert_eq!(2,  table.count());
        assert_eq!("", table.pending());
        let size_a = table.size();
        assert!(size_a > 0);

        table.push_pending_str("Hel");
        table.push_pending('l');
        table.push_pending('o');
        let b_id = table.intern_pending();

        assert_eq!(StringId(1), b_id);
        assert_eq!(2,  table.count());
        assert_eq!("", table.pending());
        assert!(table.size() == size_a);

        table.push_pending_str("oll");
        table.push_pending('e');
        table.push_pending('H');
        let c_id = table.intern_pending();

        assert_eq!(StringId(2), c_id);
        assert_eq!(3,  table.count());
        assert_eq!("", table.pending());
        assert!(table.size() > size_a);

        assert_eq!("Hello", table.get(b_id) );
        assert_eq!("olleH", table.get(c_id) );
    }

    #[test]
    fn intern_str() {
        let mut table = StringTable::new();

        let a_str = &A[2..];
        let b_str = &B[2..];
        let c_str = &C[2..];
        assert_ne!(a_str.as_ptr(), b_str.as_ptr());

        let a_id = table.intern_str(a_str);

        assert_eq!(StringId(1), a_id);
        assert_eq!(2,  table.count());
        assert_eq!("", table.pending());
        let size_a = table.size();
        assert!(size_a > 0);

        let b_id = table.intern_str(b_str);

        assert_eq!(StringId(1), b_id);
        assert_eq!(2,  table.count());
        assert_eq!("", table.pending());
        assert!(table.size() == size_a);

        let c_id = table.intern_str(c_str);

        assert_eq!(StringId(2), c_id);
        assert_eq!(3,  table.count());
        assert_eq!("", table.pending());
        assert!(table.size() > size_a);

        assert_eq!("Hello", table.get(b_id) );
        assert_eq!("olleH", table.get(c_id) );
    }
}

