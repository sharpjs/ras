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

//! Assembly language value traits.

use std::any::Any;
use std::fmt::Debug;
use super::Error;

/// Trait for values.
pub trait Value: Any + Debug + AsRef<dyn Any> + AsMut<dyn Any> {
    /// Returns the name of the value's type.
    fn type_name(&self) -> &str;

    fn eq(&self, other: &dyn Value) -> bool;

    fn clone(&self) -> Box<dyn Value>;

    // Unary
    fn op_pos (self: Box<Self>) -> Box<dyn Value> { Box::new(Error()) }
    fn op_neg (self: Box<Self>) -> Box<dyn Value> { Box::new(Error()) }
    fn op_cpl (self: Box<Self>) -> Box<dyn Value> { Box::new(Error()) }
    fn op_not (self: Box<Self>) -> Box<dyn Value> { Box::new(Error()) }

    // Exponentiative
    fn op_pow (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }

    // Multiplicative
    fn op_mul (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }
    fn op_div (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }
    fn op_mod (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }

    // Additive
    fn op_add (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }
    fn op_sub (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }

    // Shift
    fn op_shl (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }
    fn op_shr (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }
    fn op_rol (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }
    fn op_ror (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }

    // Bitwise AND/OR/XOR
    fn op_and (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }
    fn op_xor (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }
    fn op_or  (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }

    // Comparison
    fn op_eq  (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }
    fn op_ne  (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }
    fn op_lt  (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }
    fn op_le  (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }
    fn op_gt  (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }
    fn op_ge  (self: Box<Self>, _rhs: &dyn Value) -> Box<dyn Value> { Box::new(Error()) }

    // Cast
    fn op_as  (self: Box<Self>, _type: &usize) -> Box<dyn Value> { Box::new(Error()) }
    fn op_to  (self: Box<Self>, _type: &usize) -> Box<dyn Value> { Box::new(Error()) }

    // Member Access
    fn op_mem (self: Box<Self>, _name: &str) -> Box<dyn Value> { Error::new() }
}

macro_rules! impl_value_cast {
    ($type:ty: $as_type_ref:ident, $as_type_mut:ident) => {
        impl AsRef<dyn ::std::any::Any> for $type {
            #[inline(always)]
            fn as_ref(&self) -> &dyn ::std::any::Any { self }
        }

        impl AsMut<dyn ::std::any::Any> for $type {
            #[inline(always)]
            fn as_mut(&mut self) -> &mut dyn ::std::any::Any { self }
        }

        impl dyn $crate::value::Value {
            #[inline]
            pub fn $as_type_ref(&self) -> Option<&$type> {
                self.as_ref().downcast_ref::<$type>()
            }

            #[inline]
            pub fn $as_type_mut(&mut self) -> Option<&mut $type> {
                self.as_mut().downcast_mut::<$type>()
            }
        }
    }
}

/*
// These are not needed currently, but might be handy later on.
#[cfg(maybe_later)]
impl dyn Value {
    #[inline]
    pub fn downcast_ref<T: Value>(&self) -> Option<&T> {
        self.as_ref().downcast_ref::<T>()
    }

    #[inline]
    pub fn downcast_mut<T: Value>(&mut self) -> Option<&mut T> {
        self.as_mut().downcast_mut::<T>()
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Clone, Copy, Debug)]
    pub struct Unit;
    // pub required because impl_value_cast! implements public methods

    impl_value_cast!(Unit: as_unit_ref, as_unit_mut);

    impl Value for Unit {
        fn type_name(&self) -> &str {
            "unit"
        }

        fn eq(&self, other: &dyn Value) -> bool {
            other.as_unit_ref().is_some()
        }

        fn clone(&self) -> Box<dyn Value> {
            Box::new(Unit)
        }
    }

    #[test]
    fn type_name() {
        assert_eq!( Box::new(Unit).type_name(), "unit" );
    }

    #[test]
    fn op_neg() {
        assert_eq!(
            Box::new(Unit).op_neg().as_error_ref(),
            Some(&Error())
        );
    }
}
