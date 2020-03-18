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

use std::any::Any;
use super::Value;

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct Error();

impl Error {
    pub fn new() -> Box<Self> {
        Box::new(Self())
    }
}

impl dyn Value {
    pub fn as_error_ref(&self) -> Option<&Error> {
        self.downcast_ref::<Error>()
    }

    pub fn as_error_mut(&mut self) -> Option<&mut Error> {
        self.downcast_mut::<Error>()
    }
}

impl Value for Error {
    fn type_name(&self) -> &str {
        "error"
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn eq(&self, other: &dyn Value) -> bool {
        other.as_error_ref().is_some()
    }

    fn clone(&self) -> Box<dyn Value> {
        Error::new()
    }
}

