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

//! Assembly session.

use crate::name::NameTable;

// ----------------------------------------------------------------------------

/// Assembler session.
#[derive(Debug)]
pub struct Session {
    names: NameTable,
}

impl Session {
    /// Creates a new [`Session`].
    pub fn new() -> Self {
        Self {
            names: NameTable::new(),
        }
    }

    pub fn names(&self) -> &NameTable {
        &self.names
    }

    pub fn names_mut(&mut self) -> &mut NameTable {
        &mut self.names
    }
}
