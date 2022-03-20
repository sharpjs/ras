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

//! Lexical analyzer.

use std::fmt::{self, Display, Formatter};
use std::ops::Range;

use crate::num::Num;

use super::input::Cursor;

mod esc;
mod ident;
mod main;
mod memo;
mod num;
mod quoted;
mod token;

pub use self::token::*;

// ----------------------------------------------------------------------------

/// Trait for types which yield a stream of lexical tokens.
pub trait Lex {
    /// Advances to the next token and returns its type.
    fn next(&mut self) -> Token;

    /// Returns the line number at which the current token begins.
    fn line(&self) -> usize;

    /// Returns the byte position range of the current token within the input
    /// stream.
    fn range(&self) -> &Range<usize>;

    /// Returns the value of the current string-like token.
    ///
    /// If the current token is not string-like, this method is safe, but the
    /// return value is unspecified.
    fn str(&self) -> &str;

    /// Returns the value of the current character-like token.
    ///
    /// If the current token is not character-like, this method is safe, but
    /// the return value is unspecified.
    fn char(&self) -> char;

    /// Returns the value of the current integer-like token.
    ///
    /// If the current token is not integer-like, this method is safe, but the
    /// return value is unspecified.
    fn int(&self) -> u64;

    /// Returns the value of the current number-like token.
    ///
    /// If the current token is not number-like, this method is safe, but the
    /// return value is unspecified.
    fn num(&self) -> &Num;
}

// ----------------------------------------------------------------------------

/// Lexical analyzer.  Reads input and yields a stream of lexical tokens.
#[derive(Clone, Debug)]
pub struct Lexer<I: Iterator<Item = u8>> {
    input:     Cursor<I>,
    line:      usize,
    line_next: usize,
    range:     Range<usize>,
    text:      Vec<u8>,
    char:      char,
    num:       Num,
}

impl<I: Iterator<Item = u8>> Lexer<I> {
    /// Creates a new lexical analyzer for the given input iterator.
    pub fn new(iter: I) -> Self {
        let mut input = Cursor::new(iter);
        input.advance();
        Self {
            input,
            line:      0,
            line_next: 1,
            range:     Range::default(),
            text:      Vec  ::default(),
            char:      char ::default(),
            num:       Num  ::default(),
        }
    }

    /// Returns the value of the most recent token.
    pub fn value(&self, token: Token) -> Value<I> {
        Value { lexer: self, token }
    }
}

impl<I: Iterator<Item = u8>> Lex for Lexer<I> {
    #[inline]
    fn next(&mut self) -> Token {
        self.scan_main()
    }

    #[inline]
    fn line(&self) -> usize {
        self.line
    }

    #[inline]
    fn range(&self) -> &Range<usize> {
        &self.range
    }

    #[inline]
    fn str(&self) -> &str {
        // SAFETY: UTF-8 validation performed in an earlier phase.
        unsafe { std::str::from_utf8_unchecked(&self.text[..]) }
    }

    #[inline]
    fn char(&self) -> char {
        self.char
    }

    #[inline]
    fn int(&self) -> u64 {
        self.num.significand as u64
    }

    #[inline]
    fn num(&self) -> &Num {
        &self.num
    }
}

// ----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Value<'a, I: Iterator<Item = u8>> {
    lexer: &'a Lexer<I>,
    token: Token,
}

impl<'a, I: Iterator<Item = u8>> Display for Value<'a, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Token::*;
        match self.token {
            Str   => self.lexer.str().fmt(f),
            Char  => self.lexer.str().fmt(f),
            Ident => self.lexer.str().fmt(f),
            Param => self.lexer.str().fmt(f),
            Int   => self.lexer.int().fmt(f),
            Float => format!("{}", self.lexer.num()).fmt(f),
            _     => "".fmt(f),
        }
    }
}
