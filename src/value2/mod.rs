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

//! Assembly language values.

mod error;
mod integer;

pub use self::error::*;
pub use self::integer::*;

use std::borrow::Cow;

/// Assembly language value of any type.

#[derive(Clone, PartialEq, Hash, Debug)]
pub enum Value {
    Integer (Box<Integer>),
    Error   (Box<Error>),
}

impl Value {
    pub fn type_name(&self) -> &str {
        match *self {
            Self::Integer (_) => "integer",
            Self::Error   (_) => "error",
        }
    }

    pub fn op_neg(self) -> Value {
        match self {
            Self::Integer (i) => i.op_neg(),
            _ => self,
        }
    }

    pub fn op_add(self, rhs: Cow<Value>) -> Value {
        match self {
            Self::Integer (i) => i.op_add(rhs),
            _ => self,
        }
    }
}
