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

//! Integer values.

use std::any::Any;

use crate::num::Base;
use super::Value;

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct Integer {
    val:  rug::Integer, // value
    base: Base,         // preferred output base
}

impl Integer {
    pub fn new(base: Base) -> Box<Self> {
        Box::new(Self { val: rug::Integer::new(), base })
    }
}

impl dyn Value {
    pub fn as_integer_ref(&self) -> Option<&Integer> {
        self.downcast_ref::<Integer>()
    }

    pub fn as_integer_mut(&mut self) -> Option<&mut Integer> {
        self.downcast_mut::<Integer>()
    }
}

impl Value for Integer {
    fn type_name(&self) -> &str {
        "integer"
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn eq(&self, other: &dyn Value) -> bool {
        match other.as_integer_ref() {
            Some(i) => self.val == i.val,
            None    => false
        }
    }

    fn clone(&self) -> Box<dyn Value> {
        Box::new(Clone::clone(self))
    }
}
