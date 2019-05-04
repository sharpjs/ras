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

/// A lexical token.
#[derive(PartialEq, Eq, Debug)]
pub enum Token {

    // === Identifiers & Literals ===

    /// An identifier.
    /// The name is represented as an index into a string table.
    Ident(u32),

    /// A label.
    /// The name is represented as an index into a string table.
    /// Visibility is represented as Boolean value:
    ///   `false` for module-scoped, `true` for global.
    Label(u32, bool),

    /// A macro parameter.
    /// The name is represented as an index into a string table.
    Param(u32),

    /// An integer literal.
    /// The value is represented as an index into an integer table.
    Int(u32),

    /// A floating-point literal.
    /// The value is represented as an index into a float table.
    Float(u32),

    /// A string literal.
    /// The value is represented as an index into a string table.
    Str(u32),

    /// A character literal.
    /// The value is represented directly within the token.
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

