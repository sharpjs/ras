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

//! Program entry point and crate root.

#![allow(dead_code)]
#![allow(unused_macros)]

use std::env::args;
use std::fs::read_to_string;

use lang::input::{LogicalChar, Cursor};

mod lang;

fn main() {
    for path in args().skip(1) {
        let content = match read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}: {}", &path, e);
                continue;
            },
        };

        let mut cursor = Cursor::new(content.bytes());

        loop {
            cursor.advance();
            let (class, byte) = cursor.classify(&CHARS);
            println!("{:02X}: {:?}", byte, class);
            if class == Eof { break; }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Char { Lc, Uc, Etc, Non, Eof }
use Char::*;

impl LogicalChar for Char {
    const NON_ASCII: Self = Non;
    const EOF:       Self = Eof;
}

/// Mapping of 7-bit ASCII to logical characters.
static CHARS: [Char; 128] = {
    const __: Char = Etc;
[
//  x0  x1  x2  x3  x4  x5  x6  x7
//  x8  x9  xA  xB  xC  xD  xE  xF
    __, __, __, __, __, __, __, __, // 0x │········│
    __, __, __, __, __, __, __, __, // 0x │·tn··r··│
    __, __, __, __, __, __, __, __, // 1x │········│
    __, __, __, __, __, __, __, __, // 1x │········│
    __, __, __, __, __, __, __, __, // 2x │ !"#$%&'│
    __, __, __, __, __, __, __, __, // 2x │()*+,-./│
    __, __, __, __, __, __, __, __, // 3x │01234567│
    __, __, __, __, __, __, __, __, // 3x │89:;<=>?│
    __, Uc, Uc, Uc, Uc, Uc, Uc, Uc, // 4x │@ABCDEFG│
    Uc, Uc, Uc, Uc, Uc, Uc, Uc, Uc, // 4x │HIJKLMNO│
    Uc, Uc, Uc, Uc, Uc, Uc, Uc, Uc, // 5x │PQRSTUVW│
    Uc, Uc, Uc, __, __, __, __, __, // 5x │XYZ[\]^_│
    __, Lc, Lc, Lc, Lc, Lc, Lc, Lc, // 6x │`abcdefg│
    Lc, Lc, Lc, Lc, Lc, Lc, Lc, Lc, // 6x │hijklmno│
    Lc, Lc, Lc, Lc, Lc, Lc, Lc, Lc, // 7x │pqrstuvw│
    Lc, Lc, Lc, __, __, __, __, __, // 7x │xyz{|}~░│
]};
