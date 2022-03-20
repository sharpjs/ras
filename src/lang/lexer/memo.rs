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

//! Lexical analysis memoization.

use std::cmp::min;
use std::iter::Copied;
use std::ops::Range;
use std::slice::Iter;

use crate::num::Num;

use super::{Lex, Token};

// ----------------------------------------------------------------------------

/// Memoized sequence of tokens.
#[derive(Clone, Debug)]
pub struct Memo {
    line_range: Range<usize>,
    byte_range: Range<usize>,
    items:      Vec<Item>,
    values:     Vec<u64>,
}

impl Memo {
    /// Constructs a new, empty `Memo` with the given initial `line` and byte
    /// `offset`.
    pub fn new(line: usize, offset: usize) -> Self {
        Self {
            line_range: line   .. line,
            byte_range: offset .. offset,
            items:      vec![],
            values:     vec![],
        }
    }

    /// Appends the given `token` to the memo, capturing the token's line, byte
    /// range, and value from `src`.
    ///
    /// Panics if the line or byte range of the token is less than the maximum
    pub fn push<L: Lex>(&mut self, token: Token, src: &L) {
        const NEXT_LINE: usize = 1;
        const SAME_LINE: usize = 0;

        // Compute distances
        let mut eols = src.line().checked_sub(self.line_range.end)
            .expect("attempted to memoize token from earlier line");
        let mut skip = src.range().start.checked_sub(self.byte_range.end)
            .expect("attempted to memoize token from earlier byte offset");
        let mut len = src.range().len();

        // Update memo position
        self.line_range.end = src.line();
        self.byte_range.end = src.range().end;

        // Skip bytes and lines
        while eols > 0 {
            let n = Item::chunk(skip);
            self.items.push(Item::skipper(NEXT_LINE, n));
            eols -= 1;
            skip -= n;
        }

        // Skip bytes
        while skip > 1 {
            let n = Item::chunk(skip);
            self.items.push(Item::skipper(SAME_LINE, n));
            skip -= n;
        }

        // Start token
        let n = Item::chunk(len);
        self.items.push(Item::starter(skip, n));
        len -= n;

        // Lengthen token
        while len > 0 {
            let n = Item::chunk(len);
            self.items.push(Item::skipper(SAME_LINE, n));
            len -= n;
        }

        // Yield token
        self.items.push(Item::yielder(token));
    }

    /// Returns a cursor to enumerate the tokens in the memo.
    pub fn tokens(&self) -> impl Lex + '_ {
        Tokens {
            seq:    self.items.iter().copied(),
            line:   self.line_range.start,
            range:  self.byte_range.start .. self.byte_range.start,
            int:    u64   ::default(),
            char:   char  ::default(),
            float:  Num   ::default(),
            string: String::default(),
        }
    }

    #[cfg(test)]
    fn internal_representation(&self) -> &[u8] {
        use std::mem::transmute;
        unsafe { transmute(self.items.as_slice()) }
    }
}

#[derive(Clone, Copy, Debug)]
struct Item (u8);

// NAME     ENCODING  ACTION
// yielder  0ttttttt  yield token t at line, bytes start..pos
// starter  10axxxxx  pos  += a; start = pos; pos += x;
// skipper  11axxxxx  line += a;              pos += x;

impl Item {
    const MAX_LEN: usize = 0x1F;

    #[inline]
    fn chunk(len: usize) -> usize {
        min(len, Self::MAX_LEN)
    }

    #[inline]
    const fn yielder(t: Token) -> Self {
        Self(t as u8)
    }

    #[inline]
    const fn starter(skip: usize, len: usize) -> Self {
        debug_assert!(skip <= 1);
        debug_assert!(len <= Self::MAX_LEN);
        Self(0b10 << 6 | (skip as u8) << 5 | len as u8)
    }

    #[inline]
    const fn skipper(eol: usize, len: usize) -> Self {
        debug_assert!(eol <= 1);
        debug_assert!(len <= Self::MAX_LEN);
        Self(0b11 << 6 | (eol as u8) << 5 | len as u8)
    }

    #[inline]
    fn token(self) -> Option<Token> {
        use std::mem::transmute;
        if (self.0 as i8) >= 0 {
            eprintln!("self.0 = {}", self.0);
            // SAFETY: Token validity enforced by `Item::yielder()`.
            Some(unsafe { transmute(self.0) })
        } else {
            None
        }
    }

    #[inline]
    const fn mask(self) -> usize {
        // mask: all bits opposite of item bit 6
        ((self.0 as i8 >> 6) + 1) as isize as usize
    }

    #[inline]
    const fn skip(self) -> usize {
        ((self.0 >> 5) & 1) as usize
    }

    #[inline]
    const fn len(self) -> usize {
        (self.0 & 0x1F) as usize
    }
}

#[derive(Clone, Debug)]
struct Tokens<'a> {
    seq:    Copied<Iter<'a, Item>>,
    line:   usize,
    range:  Range<usize>,
    int:    u64,
    char:   char,
    float:  Num,
    string: String,
}

impl Lex for Tokens<'_> {
    fn next(&mut self) -> Token {
        let mut line   = self.line;
        let mut start  = self.range.end;
        let mut offset = start;

        let token = loop {
            let item = if let Some(item) = self.seq.next() {
                item
            } else {
                break Token::Eof
            };

            if let Some(token) = item.token() {
                break token;
            }

            let (mask, skip, len) = (item.mask(), item.skip(), item.len());

            // ENCODING  MASK  OFFSET  START    OFFSET  LINE
            // 10axxxxx  11…1  +=a     =offset  +=x     .
            // 11axxxxx  00…0  .       .        +=x     +=a

            offset += skip &  mask;
            start   = (offset ^ start) & mask ^ start;
            offset += len;
            line   += skip & !mask;
        };

        self.line  = line;
        self.range = start..offset;
        token
    }

    fn line(&self) -> usize {
        self.line
    }

    fn range(&self) -> &Range<usize> {
        &self.range
    }

    fn str(&self) -> &str {
        &self.string[..]
    }

    fn char(&self) -> char {
        self.char
    }

    fn int(&self) -> u64 {
        self.int
    }

    fn num(&self) -> &Num {
        &self.float
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct FakeLex (Token, usize, Range<usize>);

    impl Lex for FakeLex {
        fn next (&mut self) -> Token         {  self.0 }
        fn line (&self)     -> usize         {  self.1 }
        fn range(&self)     -> &Range<usize> { &self.2 }
        fn str  (&self)     -> &str          { todo!() }
        fn char (&self)     -> char          { todo!() }
        fn int  (&self)     -> u64           { todo!() }
        fn num  (&self)     -> &Num          { todo!() }
    }

    #[test]
    fn empty() {
        let memo = Memo::new(10, 1000);

        assert_eq!(memo.internal_representation(), &[]);

        let mut tokens = memo.tokens();

        assert_eq!( tokens.line(),  10);
        assert_eq!(*tokens.range(), 1000..1000);

        assert_eq!( tokens.next(),  Token::Eof);
        assert_eq!( tokens.line(),  10);
        assert_eq!(*tokens.range(), 1000..1000);
    }

    #[test]
    fn single() {
        let mut memo = Memo::new(10, 1000);

        memo.push(Token::Add, &FakeLex(Token::Add, 10, 1000..1001));

        assert_eq!(memo.internal_representation(), &[
            0b10_0_00001,
            Token::Add as u8
        ]);

        let mut tokens = memo.tokens();

        assert_eq!( tokens.line(),  10);
        assert_eq!(*tokens.range(), 1000..1000);

        assert_eq!( tokens.next(),  Token::Add);
        assert_eq!( tokens.line(),  10);
        assert_eq!(*tokens.range(), 1000..1001);

        assert_eq!( tokens.next(),  Token::Eof);
        assert_eq!( tokens.line(),  10);
        assert_eq!(*tokens.range(), 1001..1001);
    }

    #[test]
    fn complex() {
        let mut memo = Memo::new(10, 1000);

        memo.push(Token::Add, &FakeLex(Token::Add, 11, 1032..1070)); // a mythical wide `+`
        memo.push(Token::Sub, &FakeLex(Token::Sub, 13, 1136..1137));

        assert_eq!(memo.internal_representation(), &[
            //            //                              LINE  START  OFFSET
            //            // initial state             =>   10   1000    1000
            0b11_1_11111, // +1 line,        +31 bytes =>   11   1000    1031
            0b10_1_11111, // +1 byte, start, +31 bytes =>   11   1032    1063
            0b11_0_00111, //                 +07 bytes =>   11   1032    1070
            Token::Add as u8,
            0b11_1_11111, // +1 line,        +31 bytes =>   12   1032    1101
            0b11_1_11111, // +1 line,        +31 bytes =>   13   1032    1132
            0b11_0_00100, //                 +04 bytes =>   13   1032    1136
            0b10_0_00001, //          start, +01 byte  =>   13   1136    1137
            Token::Sub as u8,
        ]);

        let mut tokens = memo.tokens();

        assert_eq!( tokens.line(),  10);
        assert_eq!(*tokens.range(), 1000..1000);

        assert_eq!( tokens.next(),  Token::Add);
        assert_eq!( tokens.line(),  11);
        assert_eq!(*tokens.range(), 1032..1070);

        assert_eq!( tokens.next(),  Token::Sub);
        assert_eq!( tokens.line(),  13);
        assert_eq!(*tokens.range(), 1136..1137);

        assert_eq!( tokens.next(),  Token::Eof);
        assert_eq!( tokens.line(),  13);
        assert_eq!(*tokens.range(), 1137..1137);
    }
}
