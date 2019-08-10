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

/// Lexical token type.
#[derive(PartialEq, Debug)]
pub enum Token {
    // === Identifiers & Literals ===

    /// Identifier.
    Ident,                              // name

    /// Label.
    Label,                              // name, visibility

    /// Macro parameter.
    Param,                              // name

    /// Integer literal.
    Int,                                // value (sign, significand)

    /// Floating-point literal.
    Float,                              // value (sign, significand, exponent)

    /// String literal.
    Str,                                // value, encoding

    /// Character literal.
    Char,                               // value, encoding

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

    /// `||=` - logical OR-assignment operator.
    LogOrAssign,

    // === Punctuation ===

    /// `{` - left curly brace.
    BraceL,

    /// `}` - right curly brace.
    BraceR,

    /// `(` - left parenthesis.
    ParenL,

    /// `)` - right parenthesis.
    ParenR,

    /// `[` - left square bracket.
    BracketL,

    /// `]` - right square bracket.
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

    /// Lexical error.
    Error
}

