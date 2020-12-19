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

pub mod lexer;
pub mod token;

/// Symbol visibility and binding behaviors.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Visibility {
    /// The symbol is visible in the defining object file only.
    /// Equivalent to ELF binding STB_LOCAL.
    Local,

    /// The symbol is visible in all object files and cannot be preempted.
    /// Equivalent to ELF binding STB_GLOBAL.
    Global,

    /// The symbol is visible in all object files and can be preempted.
    /// Equivalent to ELF binding STB_WEAK.
    Weak,

    /// The symbol is visible in all object files and *MUST* be preempted.
    /// Equivalent to ELF binding STB_WEAK with an undefined symbol.
    Extern,
}
