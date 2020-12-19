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

use super::Value;

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct Error();

impl Error {
    pub fn new() -> Box<Self> {
        Box::new(Self())
    }
}

impl_value_cast!(Error: as_error_ref, as_error_mut);

impl Value for Error {
    fn type_name(&self) -> &str {
        "error"
    }

    fn eq(&self, other: &dyn Value) -> bool {
        other.as_error_ref().is_some()
    }

    fn clone(&self) -> Box<dyn Value> {
        Error::new()
    }
}
