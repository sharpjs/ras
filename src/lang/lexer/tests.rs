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
