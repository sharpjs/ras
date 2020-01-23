// This file is part of ras, an assembler.
// Copyright (C) 2020 Jeffrey Sharp
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

// NOTES:
//
// - The lexer implementation is inspired by the article "Some Strategies For
//   Fast Lexical Analysis when Parsing Programming Languages" by Sean Barrett.
//   http://nothings.org/computer/lexing.html
//
// - The term "logical character" in this file is preferred over the probably
//   more-correct term "character equivalence class".

#[cfg(OLD)] mod num;

mod core;
mod reader;

use self::reader::Reader;

// ---------------------------------------------------------------------------- 

/// Lexer states.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    /// Normal state.  Any token is possible.
    Normal,

    /// At the begining of a line.  Any token is possible.
    Bol,

    /// After a carriage return (0x0D).
    AfterCr,

    /// In a comment.
    Comment,
}

impl State {
    /// Count of lexer states.
    const COUNT: usize = Self::Comment as usize + 1;
}

// ---------------------------------------------------------------------------- 

/// Lexical analyzer.  Reads input and yields a stream of lexical tokens.
#[derive(Debug)]
pub struct Lexer<'a> {
    input: Reader<'a>,
    state: State,
    line:  usize,
    len:   usize,
}

impl<'a> Lexer<'a> {
    /// Creates a lexical analyzer that takes as input the contents of the
    /// given slice of bytes.
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input: Reader::new(input),
            state: State::Bol,
            line:  1,
            len:   0,
        }
    }

    /// Returns the source line number (1-indexed) of the current token.
    #[inline]
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the source text of the current token.
    #[inline]
    pub fn text(&self) -> &'a [u8] {
        self.input.preceding(self.len)
    }
}

// ----------------------------------------------------------------------------

#[cfg(OLD)]//#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_empty() {
        let mut lexer = Lexer::new(b"");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_unrecognized() {
        let mut lexer = Lexer::new(b"`");

        assert_eq!( lexer.next(), Token::Error );
    }

    #[test]
    fn lexer_space() {
        let mut lexer = Lexer::new(b" \t \t");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_comment() {
        let mut lexer = Lexer::new(b"# this is a comment");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_cr() {
        let mut lexer = Lexer::new(b"\r\r # hello");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_lf() {
        let mut lexer = Lexer::new(b"\n\n # hello");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_crlf() {
        let mut lexer = Lexer::new(b"\r\n\r\n # hello");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_parens() {
        let mut lexer = Lexer::new(b"()#c\n\n");

        assert_eq!( lexer.next(), Token::ParenL );
        assert_eq!( lexer.next(), Token::ParenR );
        assert_eq!( lexer.next(), Token::Eos    );
        assert_eq!( lexer.next(), Token::Eof    );
    }
}
