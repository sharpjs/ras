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

use self::num::NumData;

use super::input::Cursor;

mod esc;
mod ident;
mod main;
mod num;
mod quoted;

// ----------------------------------------------------------------------------

/// Lexical tokens.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Token {
    // === Identifiers & Literals ===

    /// Identifier.
    Ident,                              // props: name

    /// Macro parameter.
    Param,                              // props: name

    /// Integer literal.
    Int,                                // props: sign, significand

    /// Floating-point literal.
    Float,                              // props: sign, significand, exponent

    /// String literal.
    Str,                                // props: content, encoding

    /// Character literal.
    Char,                               // props: content, encoding

    // === Operators ===

    /// `@` - alias operator.
    Alias,

    /// `~` - bitwise NOT operator.
    BitNot,

    /// `!` - logical NOT operator, side-effect operator.
    LogNot,

    /// `++` - increment operator.
    Inc,

    /// `--` - decrement operator.
    Dec,

    /// `*` - multiplication operator.
    Mul,

    /// `/` - division operator.
    Div,

    /// `%` - modulo operator, treat-as-unsigned operator.
    Mod,

    /// `+` - addition operator, treat-as-signed operator.
    Add,

    /// `-` - subtraction operator, negation operator.
    Sub,

    /// `<<` - left shift operator.
    Shl,

    /// `>>` - signed right shift operator.
    Shr,

    /// `&` - bitwise AND operator.
    BitAnd,

    /// `^` - bitwise XOR operator.
    BitXor,

    /// `|` - bitwise OR operator.
    BitOr,

    /// `==` - equal-to operator.
    Eq,

    /// `!=` - not-equal-to operator.
    NotEq,

    /// `<` - less-than operator.
    Less,

    /// `>` - greater-than operator.
    More,

    /// `<=` - less-than-or-equal-to operator.
    LessEq,

    /// `>=` - greater-than-or-equal-to operator.
    MoreEq,

    /// `?` - not-known indicator.
    Unknown,

    /// `&&` - logical AND operator.
    LogAnd,

    /// `^^` - logical XOR operator.
    LogXor,

    /// `||` - logical OR operator.
    LogOr,

    /// `=` - assignment operator.
    Assign,

    /// `*=` - compound multiplication-assignment operator.
    MulAssign,

    /// `/=` - compound division-assignment operator.
    DivAssign,

    /// `%=` - compound modulo-assignment operator.
    ModAssign,

    /// `+=` - compound addition-assigment operator.
    AddAssign,

    /// `-=` - compound subtraction-assignment operator.
    SubAssign,

    /// `<<=` - compound left-shift-assignment operator.
    ShlAssign,

    /// `>>=` - compound right-shift-assignment operator.
    ShrAssign,

    /// `&=` - compound bitwise-AND-assignment operator.
    BitAndAssign,

    /// `^=` - compound bitwise-XOR-assignment operator.
    BitXorAssign,

    /// `|=` - compound bitwise-OR-assignment operator.
    BitOrAssign,

    /// `&&=` - compound logical-AND-assignment operator.
    LogAndAssign,

    /// `^^=` - compound logical-XOR-assignment operator.
    LogXorAssign,

    /// `||=` - compound logical-OR-assignment operator.
    LogOrAssign,

    // === Punctuation ===

    /// `{` - left curly brace.
    LCurly,

    /// `}` - right curly brace.
    RCurly,

    /// `(` - left parenthesis.
    LParen,

    /// `)` - right parenthesis.
    RParen,

    /// `[` - left square bracket.
    LSquare,

    /// `]` - right square bracket.
    RSquare,

    /// `:` - private label declarator, identifier composition operator.
    Colon,

    /// `+:` - implicit-signed operator.
    Signed,

    /// `%:` - implicit-unsigned operator.
    Unsigned,

    /// `::` - public label declarator.
    Public,

    /// `:?` - weak label declarator.
    Weak,

    /// `,` - item separator.
    Comma,

    // === Terminators ===

    /// End of statement.
    Eos,

    /// End of file.
    Eof
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Token::*;

        let s = match *self {
            Ident        => "ident",
            Param        => "param",
            Int          => "int",
            Float        => "float",
            Str          => "str",
            Char         => "char",
            Alias        => "@",
            BitNot       => "~",
            LogNot       => "!",
            Inc          => "++",
            Dec          => "--",
            Mul          => "*",
            Div          => "/",
            Mod          => "%",
            Add          => "+",
            Sub          => "-",
            Shl          => "<<",
            Shr          => ">>",
            BitAnd       => "&",
            BitXor       => "^",
            BitOr        => "|",
            Eq           => "==",
            NotEq        => "!=",
            Less         => "<",
            More         => ">",
            LessEq       => "<=",
            MoreEq       => ">=",
            Unknown      => "?",
            LogAnd       => "&&",
            LogXor       => "^^",
            LogOr        => "||",
            Assign       => "=",
            MulAssign    => "*=",
            DivAssign    => "/=",
            ModAssign    => "%=",
            AddAssign    => "+=",
            SubAssign    => "-=",
            ShlAssign    => "<<=",
            ShrAssign    => ">>=",
            BitAndAssign => "&=",
            BitXorAssign => "^=",
            BitOrAssign  => "|=",
            LogAndAssign => "&&=",
            LogXorAssign => "^^=",
            LogOrAssign  => "||=",
            LCurly       => "{",
            RCurly       => "}",
            LParen       => "(",
            RParen       => ")",
            LSquare      => "[",
            RSquare      => "]",
            Signed       => "+:",
            Unsigned     => "%:",
            Colon        => ":",
            Weak         => ":?",
            Public       => "::",
            Comma        => ",",
            Eos          => "EOS",
            Eof          => "EOF",
        };
        s.fmt(f)
    }
}

// ----------------------------------------------------------------------------

/// Trait for types which yield a stream of lexical tokens.
pub trait Lex {
    /// Advances to the next token and returns its type.
    fn next(&mut self) -> Token;

    /// Returns the type of the current token.
    fn token(&self) -> Token;

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
    fn num(&self) -> &NumData;
}

// ----------------------------------------------------------------------------

/// Lexical analyzer.  Reads input and yields a stream of lexical tokens.
#[derive(Clone, Debug)]
pub struct Lexer<I: Iterator<Item = u8>> {
    input:     Cursor<I>,
    token:     Token,
    line:      usize,
    line_next: usize,
    range:     Range<usize>,
    str_buf:   Vec<u8>,
    num:       NumData,
}

impl<I: Iterator<Item = u8>> Lexer<I> {
    /// Creates a new lexical analyzer for the given input iterator.
    pub fn new(iter: I) -> Self {
        let mut input = Cursor::new(iter);
        input.advance();
        Self {
            input, token: Token::Eof, line: 0, line_next: 1, range: 0..0,
            str_buf: vec![],
            num:  NumData::default(),
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
        let token = self.scan_main();
        self.token = token;
        token
    }

    #[inline]
    fn token(&self) -> Token {
        self.token
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
        unsafe { std::str::from_utf8_unchecked(&self.str_buf[..]) }
    }

    #[inline]
    fn char(&self) -> char {
        char::from_u32(self.num.significand as u32).unwrap_or_default()
    }

    #[inline]
    fn int(&self) -> u64 {
        self.num.significand
    }

    #[inline]
    fn num(&self) -> &NumData {
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
