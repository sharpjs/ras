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

//! Interned names.

use std::collections::HashMap;
use std::mem;
use std::ops::Index;

/// Predicted length in bytes of an average name in a [`NameTable`].  This is
/// used only to tune [`INITIAL_NAME_CAPACITY`].
const AVERAGE_NAME_LENGTH: usize = 8;

/// Length in bytes at which a name becomes 'extremely' long.  [`StringArena`]
/// stores such a name in a separate buffer if the name will not fit into the
/// current open buffer.
const EXTREME_NAME_LENGTH: usize = 256;

/// Capacity of each buffer in a [`StringArena`].
const NAME_BUFFER_CAPACITY: usize = 4096;

/// Initial capacity of the internal collections of a [`NameTable`].
const INITIAL_NAME_CAPACITY: usize = NAME_BUFFER_CAPACITY / AVERAGE_NAME_LENGTH;

// ----------------------------------------------------------------------------

/// Interned name.
///
/// A `Name` is an opaque structure that uniquely identifies a name in a
/// [`NameTable`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Name(u32);

// ----------------------------------------------------------------------------

/// Table of interned names.
#[derive(Debug)]
pub struct NameTable {
    vec: Vec     <&'static str>,
    map: HashMap <&'static str, Name>,
    mem: StringArena,
}

impl NameTable {
    /// Creates an new, empty [`NameTable`].
    pub fn empty() -> NameTable {
        Self {
            vec: Vec        ::with_capacity(INITIAL_NAME_CAPACITY),
            map: HashMap    ::with_capacity(INITIAL_NAME_CAPACITY),
            mem: StringArena::new(),
        }
    }

    /// Returns the number of names in the table.
    #[inline]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns a reference to the string value of the given `name`.
    ///
    /// If `name` originates from a different name table, the method is safe,
    /// but the returned reference is unspecified.
    #[inline]
    pub fn get(&self, name: Name) -> &str {
        self.vec.get(name.0 as usize).copied().unwrap_or_default()
    }

    /// Copies the given string into the table if not already present, and
    /// returns the [`Name`] representing the string.
    pub fn add(&mut self, str: &str) -> Name {
        if let Some(&name) = self.map.get(str) {
            name
        } else {
            // Store the string
            let str = self.mem.add(str);

            // Promote lifetime to 'static
            // SAFETY: String is alive and immobile for the remainder of 'self,
            // and get() returns references constrained to a shorter lifetime.
            let str: &'static str = unsafe { mem::transmute(str) };

            // Convert to Name and store
            let name = Name(self.vec.len() as u32);
            self.vec.push(str);
            self.map.insert(str, name);
            name
        }
    }
}

impl Index<Name> for NameTable {
    type Output = str;

    #[inline]
    fn index(&self, index: Name) -> &Self::Output {
        self.get(index)
    }
}

// ----------------------------------------------------------------------------

/// Arena allocator specialized for names.
#[derive(Debug)]
struct StringArena {
    /// Buffer being filled by new names.
    open: String,

    /// Buffers previously filled with names.
    full: Vec<String>,
}

impl StringArena {
    /// Creates a new, empty [`StringArena`].
    fn new() -> Self {
        Self {
            open: String ::with_capacity(NAME_BUFFER_CAPACITY),
            full: Vec    ::new(),
        }
    }

    /// Copies the given string `str` into the arena and returns a reference to the copy.
    fn add(&mut self, str: &str) -> &str {
        let need = str.len();
        let room = self.open.capacity() - self.open.len();

        if need <= room {
            // There is room in the open buffer
        } else if need < EXTREME_NAME_LENGTH {
            // The open buffer is full; switch to another
            let open = String::with_capacity(NAME_BUFFER_CAPACITY);
            let full = mem::replace(&mut self.open, open);
            self.full.push(full);
        } else {
            // Long string gets its own allocation
            self.full.push(str.to_string());
            return self.full.last().unwrap().as_str()
        }

        // Copy into the open buffer
        let buf = &mut self.open;
        let idx = buf.len();
        buf.push_str(str);

        // Return reference into open buffer
        &buf[idx..]
    }
}

// ----------------------------------------------------------------------------

// $($id:ident => $val:literal)*,
macro_rules! prepopulate {
    ($($(#[$attr:meta])* $ident:ident => $value:literal,)*) => {
        #[repr(u32)]
        #[allow(non_camel_case_types)]
        enum _Names { $($ident),* }

        impl Name { $(
            $(#[$attr])*
            pub const $ident: Name = Name(_Names::$ident as u32);
        )* }

        impl NameTable {
            /// Creates a new [`NameTable`] prepopulated with common strings.
            pub fn new() -> NameTable {
                let mut table = Self::empty();
                $( table.add($value); )*
                table
            }
        }
    };
}

prepopulate! {
    /// `Name` representing the empty string.
    EMPTY       => "",

    // Literals
    B           => "b",
    B_UC        => "B",
    O           => "o",
    O_UC        => "O",
    D           => "d",
    D_UC        => "D",
    X           => "x",
    X_UC        => "X",

    // Misc
    DOT         => ".",
    DOT_NOP     => ".nop",

    // Messages
    DOT_PRINT   => ".print",
    DOT_WARNING => ".warning",
    DOT_ERROR   => ".error",

    // Inclusion
    DOT_INCLUDE => ".include",
    DOT_INCBIN  => ".incbin",

    // Macros
    DOT_DEFINE  => ".define",
    DOT_MACRO   => ".macro",
    DOT_UNDEF   => ".undef",
    DOT_UNMAC   => ".unmac",

    // Control flow
    DOT_IF      => ".if",
    DOT_ELSEIF  => ".elif",
    DOT_ELSE    => ".else",
    DOT_END     => ".end",
    DOT_FOR     => ".for",

    // Section
    DOT_SECTION => ".section",
    DOT_CODE    => ".code",
    DOT_DATA    => ".data",
    DOT_BSS     => ".bss",
    DOT_STRUCT  => ".struct",

    // Address
    DOT_ORG     => ".org",
    DOT_SKIP    => ".skip",
    DOT_ALIGN   => ".align",

    // Data
    DOT_INT8    => ".int8",
    DOT_INT16   => ".int16",
    DOT_INT32   => ".int32",
    DOT_INT64   => ".int64",
    DOT_FLOAT16 => ".float16",
    DOT_FLOAT32 => ".float32",
    DOT_FLOAT64 => ".float64",
    DOT_FLOAT96 => ".float96",
    DOT_ASCII   => ".ascii",
    DOT_ASCIIZ  => ".asciiz",
    DOT_UTF8    => ".utf8",
    DOT_UTF8Z   => ".utf8z",
    DOT_UTF16   => ".utf8",
    DOT_UTF16Z  => ".utf8z",
    DOT_NEW     => ".object",
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{Name, NameTable};

    const INITIAL_LEN: usize = 46; // Increment for each prepopulated name

    #[test]
    fn empty() {
        let t = NameTable::empty();

        assert_eq!(t.len(),                  0);
        assert_eq!(t.get(Name::EMPTY),       "");
        assert_eq!(t.get(Name::DOT_SECTION), "");
    }

    #[test]
    fn new() {
        let t = NameTable::new();

        assert_eq!(t.len(),                  INITIAL_LEN);
        assert_eq!(t.get(Name::EMPTY),       "");
        assert_eq!(t.get(Name::DOT_SECTION), ".section");
    }

    #[test]
    fn add_once() {
        let mut t = NameTable::new();

        let name = t.add("foo");

        assert_eq!(t.len(),              INITIAL_LEN + 1);
        assert_eq!(t.get(Name::EMPTY),   "");
        assert_eq!(t.get(Name::DOT_SECTION), ".section");
        assert_eq!(t.get(name),          "foo");
    }

    #[test]
    fn add_twice() {
        let mut t = NameTable::new();

        let name0 = t.add("foo");
        let name1 = t.add("foo");

        assert_eq!(t.len(),                  INITIAL_LEN + 1);
        assert_eq!(t.get(Name::EMPTY),       "");
        assert_eq!(t.get(Name::DOT_SECTION), ".section");
        assert_eq!(t.get(name1),             "foo");

        assert_eq!(name0, name1);
        assert_eq!(
            t.get(name0).as_ptr() as usize,
            t.get(name1).as_ptr() as usize
        );
    }
}
