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

//! Assembly language integer values.

use std::borrow::Cow;
use std::ops::*;
use rug::ops::*;
use crate::num::Base;
use super::Value;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct Integer {
    val:  rug::Integer, // value
    base: Base,         // preferred output base
}

impl Integer {
    pub fn op_neg(mut self: Box<Self>) -> Value {
        self.val.neg_assign();
        Value::Integer(self)
    }

    pub fn op_add(mut self: Box<Self>, rhs: Cow<Value>) -> Value {
        match *rhs.as_ref() {
            Value::Integer(ref i) => {
                self.val.add_assign(&i.val);
                Value::Integer(self)
            },
            Value::Error(_) => rhs.into_owned(),
        }
    }
}
