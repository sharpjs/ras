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

    // Storage for interned strings.  Once a string is stored here, those
    // characters do not change.
    chars: String,
}

impl StringTable {
    const INITIAL_STR_CAPACITY: usize =      1024;
    const INITIAL_CHR_CAPACITY: usize = 16 * 1024;

    /// Creates a new `StringTable`.  Initially, the table contains only the
    /// empty string.
    pub fn new() -> Self {
        // Create an empty table
        let mut table = Self {
            map:   HashMap::with_capacity(Self::INITIAL_STR_CAPACITY),
            table: Vec    ::with_capacity(Self::INITIAL_STR_CAPACITY),
            chars: String ::with_capacity(Self::INITIAL_CHR_CAPACITY),
        };

        // Intern the empty string so that it has id 0, thus ensuring that the
        // default StringId maps to the empty string.
        table.intern("");

        table
    }

    /// Interns a string, returning a `StringId` that can be used to retrieve
    /// the string later.
    pub fn intern(&mut self, s: &str) -> StringId {
        // Check if s is interned already
        if let Some(&id) = self.map.get(s) {
            return StringId(id)
        }

        // Copy s to interned storage
        let chars = &mut self.chars;
        let start = chars.len();
                    chars.push_str(s);
        let end   = chars.len();

        // Re-reference s from interned storage
        let s = &chars[start..end];
        let s = unsafe { transmute::<&'_ str, &'static str>(s) };

        // Add entry to table
        let table = &mut self.table;
        let index = table.len() as u32;
                    table.push((start, end));

        // Add entry to map
        self.map.insert(s, index);

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

// TODO: Tests

