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
    /// Identifier with index into string table.
    Ident(usize),

    /// Label with index into string table.
    Label(usize),

    /// Integer literal.
    Int(u64),
    // TODO: Non-two's-complement representations
    // TODO: Float and its representations

    /// Character literal.
    Char(char),

    /// String literal.
    Str(usize),

    /// Plus `+`.
    Plus,

    /// Minus `-`.
    Minus,

    // TODO: More operators

    /// Left brace `{`.
    BraceL,

    /// Right brace `}`.
    BraceR,

    /// Left parenthesis `(`.
    ParenL,

    /// Right parenthesis `)`.
    ParenR,

    /// Left bracket  `[`.
    BracketL,

    /// Right bracket `]`.
    BracketR,

    /// Comma `,`.
    Comma,

    /// End of statement.
    Eos,

    /// End of file.
    Eof,

    /// Lexical error.
    Error
}

