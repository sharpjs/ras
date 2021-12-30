// This file is part of ras, an assembler.
// Copyright 2021 Jeffrey Sharp
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
