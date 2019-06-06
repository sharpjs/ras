// This file is part of ras, an assembler.
// Copyright (C) 2019 Jeffrey Sharp
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

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub struct Integer {
    // flags:
    //   representation
    //   preferred output base
    //   internal or vectorized
    //   signedness
    //   sign
    words: Vec<usize> // 3*usize (24 bytes on 64-bit)
}

impl Integer {
    #[inline]
    pub fn negative_zero() -> Self {
        Self { words: vec![0] }
    }

    #[inline]
    pub fn is_positive_zero(&self) -> bool {
        self.words.len() == 0 
    }

    #[inline]
    pub fn is_negative_zero(&self) -> bool {
        self.words.len() == 1 && self.words[0] == 0
    }

    #[inline]
    pub fn prefer_negative_zero(&mut self) {
        if self.words.len() == 0 {
            self.words.push(0)
        }
    }

    #[inline]
    pub fn avoid_negative_zero(&mut self) {
        if self.is_negative_zero() {
            self.words.clear()
        }
    }
}

impl From<usize> for Integer {
    fn from(val: usize) -> Self {
        if val == 0 {
            Self::default()
        } else {
            Self { words: vec![val] }
        }
    }
}

