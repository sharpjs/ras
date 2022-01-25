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

//! Value-and-context wrapper.

// ----------------------------------------------------------------------------

/// Trait for wrapping values with contextual information.
///
/// A blanket implementation is provided for all [`Sized`] values.
pub trait With: Sized {
    /// Wraps the value with the given `context`.
    #[inline]
    fn with<C>(self, context: C) -> Contexted<Self, C> {
        Contexted::new(self, context)
    }
}

// Blanket implementation
impl<T> With for T { }

// ----------------------------------------------------------------------------

/// Wrapped value with contextual information.
#[derive(Clone, Copy, Debug)]
pub struct Contexted<T, C = ()> {
    /// Wrapped value.
    pub value: T,

    /// Contextual information.
    pub context: C,
}

impl<T, C> Contexted<T, C> {
    /// Creates a [`Contexted`] wrapper with the given `value` and `context`.
    #[inline]
    pub fn new(value: T, context: C) -> Self {
        Self { value, context }
    }

    /// Returns a derivative [`Contexted`] wrapper by replacing the wrapped
    /// value with the given `value`, leaving the context untouched.
    #[inline]
    pub fn sub<U>(self, value: U) -> Contexted<U, C> {
        Contexted::new(value, self.context)
    }

    /// Returns a derivative [`Contexted`] wrapper by applying a function `f`
    /// to the wrapped value, leaving the context untouched.
    #[inline]
    pub fn map<F, U>(self, f: F) -> Contexted<U, C>
    where
        F: FnOnce(T) -> U
    {
        Contexted::new(f(self.value), self.context)
    }
}
