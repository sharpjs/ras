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

use crate::mem::string_table::*;
use crate::lang::Visibility;

/// A lexical token.
#[derive(PartialEq, Debug)]
pub enum Token {

    // === Identifiers & Literals ===

    /// An identifier.
    Ident(StringId),

    /// A label.
    Label(Visibility, StringId),

    /// A macro parameter.
    Param(StringId),

    /// An integer literal in 64-bit two's-complement representation.
    Int(i64),
    // TODO: Other integer representations.

    /// A floating-point literal in 64-bit native representation.
    Float(f64),
    // TODO: Other integer representations.

    /// A string literal.
    Str(Box<String>),

    /// A character literal.
    Char(char),

    // === Operators ===
    
    /// `!` - logical NOT operator, side-effect indicator.
    LogNot,
    
    /// `~` - bitwise NOT operator.
    BitNot,

    /// `*` - signed multiplication operator.
    Mul,

    /// `/` - signed division operator.
    Div,

    /// `%` - signed modulo operator.
    Mod,

    /// `+*` - unsigned multiplication operator.
    UnsMul,

    /// `+/` - unsigned division operator.
    UnsDiv,

    /// `+%` - unsigned modulo operator.
    UnsMod,

    /// `+` - addition operator, increment indicator.
    Add,

    /// `-` - subtraction operator, negation operator, decrement indicator.
    Sub,

    /// `<<` - left shift operator.
    Shl,

    /// `>>` - signed right shift operator.
    Shr,

    /// `+>>` - unsigned right shift operator.
    UnsShr,

    /// `&` - bitwise AND operator.
    BitAnd,

    /// `^` - bitwise XOR operator.
    BitXor,

    /// `|` - bitwise OR operator.
    BitOr,

    /// `==` - equality operator.
    Equal,

    /// `!=` - inequality operator.
    NotEqual,

    /// `<` - signed less-than operator.
    Less,

    /// `>` - signed greater-than operator.
    More,

    /// `<=` - signed less-than-or-equal-to operator.
    LessEqual,

    /// `>=` - signed greater-than-or-equal-to operator.
    MoreEqual,

    /// `+<` - unsigned less-than operator.
    UnsLess,

    /// `+>` - unsigned greater-than operator.
    UnsMore,

    /// `+<=` - unsigned less-than-or-equal-to operator.
    UnsLessEqual,

    /// `+>=` - unsigned greater-than-or-equal-to operator.
    UnsMoreEqual,

    /// `?` - not-known indicator.
    Unknown,

    /// `&&` - logical AND operator.
    LogAnd,

    /// `||` - logical OR operator.
    LogOr,

    /// `=`
    Assign,

    /// `*=`
    MulAssign,

    /// `/=`
    DivAssign,

    /// `/=`
    ModAssign,

    /// `+*=`
    UnsMulAssign,

    /// `+/=`
    UnsDivAssign,

    /// `+/=`
    UnsModAssign,

    /// `+=`
    AddAssign,

    /// `-=`
    SubAssign,

    /// `<<=`
    ShlAssign,

    /// `>>=`
    ShrAssign,

    /// `+>>=`
    UnsShrAssign,

    /// `&=`
    BitAndAssign,

    /// `^=`
    BitXorAssign,

    /// `|=`
    BitOrAssign,

    /// `&&=`
    LogAndAssign,

    /// `||=`
    LogOrAssign,

    // === Punctuation ===

    /// `{` - left brace.
    BraceL,

    /// `}` - right brace.
    BraceR,

    /// `(` - left parenthesis.
    ParenL,

    /// `)` - right parenthesis.
    ParenR,

    /// `[` - left bracket.
    BracketL,

    /// `]` - right bracket.
    BracketR,

    /// `:` - item joiner.
    Colon,

    /// `,` - item separator.
    Comma,

    // === Terminators ===

    /// End of statement.
    Eos,

    /// End of file.
    Eof,

    /// A lexical error.
    Error
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn size_of_token() {
        assert_eq!(16 /*bytes*/, size_of::<Token>());
    }
}

