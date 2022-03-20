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

//! Lexical tokens.

use std::fmt::{self, Display, Formatter};

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

    /// `~` - bitwise NOT operator, range operator.
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

    /// `:` - private label declarator, join operator.
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

impl Token {
    /// Returns whether the token is an end-of-statement token.
    pub fn is_eos(self) -> bool {
        matches!(self, Self::Eos | Self::Eof)
    }
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
