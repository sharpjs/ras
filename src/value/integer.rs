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

//use std::cmp::Ordering::*;
//use rug::{Assign};
use rug::ops::*;

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

    pub fn accumulate_digit(&mut self, _dig: u8) {
    }
}

impl_value_cast!(Integer: as_integer_ref, as_integer_mut);

impl Value for Integer {
    #[inline]
    fn type_name(&self) -> &str {
        "integer"
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

    #[inline]
    fn op_pos(self: Box<Self>) -> Box<dyn Value> {
        self
    }

    #[inline]
    fn op_neg(mut self: Box<Self>) -> Box<dyn Value> {
        self.val.neg_assign();
        self
    }

    #[inline]
    fn op_cpl(mut self: Box<Self>) -> Box<dyn Value> {
        self.val.not_assign();
        self
    }

    #[inline]
    fn op_not(mut self: Box<Self>) -> Box<dyn Value> {
        self.val.signum_mut();
        self.val.abs_mut();
        self
    }
}
