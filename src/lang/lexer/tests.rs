// This file is part of ras, an assembler.
// Copyright 2020 Jeffrey Sharp
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

use crate::asm::Assembler;
use crate::lang::token::Token;
use super::*;

#[test]
fn lexer_empty() {
    let mut asm   = Assembler::new();
    let mut lexer = Lexer::new(&mut asm, "test.s", b"");

    assert_eq!( lexer.next(), Token::Eof );
}

#[test]
fn lexer_unrecognized() {
    let mut asm   = Assembler::new();
    let mut lexer = Lexer::new(&mut asm, "test.s", b"`");

    assert_eq!( lexer.next(), Token::Error );
}

#[test]
fn lexer_space() {
    let mut asm   = Assembler::new();
    let mut lexer = Lexer::new(&mut asm, "test.s", b" \t \t");

    assert_eq!( lexer.next(), Token::Eof );
}

#[test]
fn lexer_comment() {
    let mut asm   = Assembler::new();
    let mut lexer = Lexer::new(&mut asm, "test.s", b"# this is a comment");

    assert_eq!( lexer.next(), Token::Eof );
}

#[test]
fn lexer_cr() {
    let mut asm   = Assembler::new();
    let mut lexer = Lexer::new(&mut asm, "test.s", b"\r\r");

    assert_eq!( lexer.next(), Token::Error );
}

#[test]
fn lexer_lf() {
    let mut asm   = Assembler::new();
    let mut lexer = Lexer::new(&mut asm, "test.s", b"\n\n # hello");

    assert_eq!( lexer.next(), Token::Eof );
}

#[test]
fn lexer_crlf() {
    let mut asm   = Assembler::new();
    let mut lexer = Lexer::new(&mut asm, "test.s", b"\r\n\r\n # hello");

    assert_eq!( lexer.next(), Token::Eof );
}

#[test]
fn lexer_parens() {
    let mut asm   = Assembler::new();
    let mut lexer = Lexer::new(&mut asm, "test.s", b"()#c\n\n");

    assert_eq!( lexer.next(), Token::LParen );
    assert_eq!( lexer.next(), Token::RParen );
    assert_eq!( lexer.next(), Token::Eos    );
    assert_eq!( lexer.next(), Token::Eof    );
}
