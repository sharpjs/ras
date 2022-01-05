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

use super::input::Cursor;

mod main;

// ----------------------------------------------------------------------------

/// Lexical tokens.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Token {
    // === Identifiers & Literals ===

    /// Identifier.
    Ident,                              // props: name

    /// Label.
    Label,                              // props: name, visibility

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

    /// `!` - logical NOT operator, side-effect indicator.
    LogNot,

    /// `~` - bitwise NOT operator.
    BitNot,

    /// `++` increment operator.
    Inc,

    /// `--` decrement operator.
    Dec,

    /// `*` - signed multiplication operator.
    Mul,

    /// `/` - signed division operator.
    Div,

    /// `%` - signed modulo operator.
    Mod,

    /// `+*` - unsigned multiplication operator.
    UMul,

    /// `+/` - unsigned division operator.
    UDiv,

    /// `+%` - unsigned modulo operator.
    UMod,

    /// `+` - addition operator, increment indicator.
    Add,

    /// `-` - subtraction operator, negation operator, decrement indicator.
    Sub,

    /// `<<` - left shift operator.
    Shl,

    /// `>>` - signed right shift operator.
    Shr,

    /// `+>>` - unsigned right shift operator.
    UShr,

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

    /// `<` - signed less-than operator.
    Less,

    /// `>` - signed greater-than operator.
    More,

    /// `<=` - signed less-than-or-equal-to operator.
    LessEq,

    /// `>=` - signed greater-than-or-equal-to operator.
    MoreEq,

    /// `+<` - unsigned less-than operator.
    ULess,

    /// `+>` - unsigned greater-than operator.
    UMore,

    /// `+<=` - unsigned less-than-or-equal-to operator.
    ULessEq,

    /// `+>=` - unsigned greater-than-or-equal-to operator.
    UMoreEq,

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

    /// `*=` - signed multiplication-assignment operator.
    MulAssign,

    /// `/=` - signed division-assignment operator.
    DivAssign,

    /// `%=` - signed modulo-assignment operator.
    ModAssign,

    /// `+*=` - unsigned multiplication-assignment operator.
    UMulAssign,

    /// `+/=` - unsigned division-assignment operator.
    UDivAssign,

    /// `+/=` - unsigned modulo-assignment operator.
    UModAssign,

    /// `+=` - addition-assigment operator.
    AddAssign,

    /// `-=` - subtraction-assignment operator.
    SubAssign,

    /// `<<=` - left-shift-assignment operator.
    ShlAssign,

    /// `>>=` - signed right-shift-assignment operator.
    ShrAssign,

    /// `+>>=` - unsigned right-shift-assignment operator.
    UShrAssign,

    /// `&=` - bitwise AND-assignment operator.
    BitAndAssign,

    /// `^=` - bitwise XOR-assignment operator.
    BitXorAssign,

    /// `|=` - bitwise OR-assignment operator.
    BitOrAssign,

    /// `&&=` - logical AND-assignment operator.
    LogAndAssign,

    /// `^^=` - logical XOR-assignment operator.
    LogXorAssign,

    /// `||=` - logical OR-assignment operator.
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

    /// `:` - item joiner.
    Colon,

    /// `,` - item separator.
    Comma,

    // === Terminators ===

    /// End of statement.
    Eos,

    /// End of file.
    Eof
}

impl Token {
    /// Returns the specified variant of the token.
    fn variant(self, n: u8) -> Token {
        use Token::*;
        let n = n as usize & 1;
        match self  {
            Ident     => [Ident,     Param     ][n],
            Mul       => [Mul,       UMul      ][n],
            Div       => [Div,       UDiv      ][n],
            Mod       => [Mod,       UMod      ][n],
            Shr       => [Shr,       UShr      ][n],
            Less      => [Less,      ULess     ][n],
            More      => [More,      UMore     ][n],
            LessEq    => [LessEq,    ULessEq   ][n],
            MoreEq    => [MoreEq,    UMoreEq   ][n],
            MulAssign => [MulAssign, UMulAssign][n],
            DivAssign => [DivAssign, UDivAssign][n],
            ModAssign => [ModAssign, UModAssign][n],
            ShrAssign => [ShrAssign, UShrAssign][n],
            _         => self
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Token::*;

        let s = match *self {
            Ident        => "ident",
            Label        => "label",
            Param        => "param",
            Int          => "int",
            Float        => "float",
            Str          => "str",
            Char         => "char",
            LogNot       => "!",
            BitNot       => "~",
            Inc          => "++",
            Dec          => "--",
            Mul          => "*",
            Div          => "/",
            Mod          => "%",
            UMul         => "+*",
            UDiv         => "+/",
            UMod         => "+%",
            Add          => "+",
            Sub          => "-",
            Shl          => "<<",
            Shr          => ">>",
            UShr         => "+>>",
            BitAnd       => "&",
            BitXor       => "^",
            BitOr        => "|",
            Eq           => "==",
            NotEq        => "!=",
            Less         => "<",
            More         => ">",
            LessEq       => "<=",
            MoreEq       => ">=",
            ULess        => "+<",
            UMore        => "+>",
            ULessEq      => "+<=",
            UMoreEq      => "+>=",
            Unknown      => "?",
            LogAnd       => "&&",
            LogXor       => "^^",
            LogOr        => "||",
            Assign       => "=",
            MulAssign    => "*=",
            DivAssign    => "/=",
            ModAssign    => "%=",
            UMulAssign   => "+*=",
            UDivAssign   => "+/=",
            UModAssign   => "+%=",
            AddAssign    => "+=",
            SubAssign    => "-=",
            ShlAssign    => "<<=",
            ShrAssign    => ">>=",
            UShrAssign   => "+>>=",
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
            Colon        => ":",
            Comma        => ",",
            Eos          => "EOS",
            Eof          => "EOF",
        };
        s.fmt(f)
    }
}

// ----------------------------------------------------------------------------

/// Lexical analyzer.  Reads input and yields a stream of lexical tokens.
#[derive(Clone, Debug)]
pub struct Lexer<I: Iterator<Item = u8>> {
    input:     Cursor<I>,
    line:      usize,
    line_next: usize,
    range:     Range<usize>,
}

impl<I: Iterator<Item = u8>> Lexer<I> {
    /// Creates a new lexical analyzer for the given input iterator.
    pub fn new(iter: I) -> Self {
        let mut input = Cursor::new(iter);
        input.advance();
        Self { input, line: 0, line_next: 1, range: 0..0 }
    }

    /// Advances to the next token and returns its type.
    #[inline]
    pub fn next(&mut self) -> Token {
        self.scan_main()
    }

    /// Returns the line number at which the current token begins.
    #[inline]
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the byte position range of the current token within the input
    /// stream.
    #[inline]
    pub fn range(&self) -> &Range<usize> {
        &self.range
    }
}
