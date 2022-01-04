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

//! Program entry point and crate root.

#![allow(dead_code)]
#![allow(unused_macros)]

mod lang;

use std::env::args;
use std::fs::File;
use std::io::{Read, stdin};

use lang::lexer::{Lexer, Token};

fn main() {
    let mut content = String::with_capacity(4096);

    for path in args().skip(1) {
        content.clear();

        let result = if path == "-" {
            stdin().read_to_string(&mut content)
        } else {
            File::open(&path).and_then(|mut f| f.read_to_string(&mut content))
        };

        if let Err(e) = result {
            eprintln!("{}: {}", path, e);
            continue;
        }

        let mut lexer = Lexer::new(content.bytes());

        loop {
            let token = lexer.next();
            println!(
                "[{}:{}] @{} +{} {:?}",
                path,
                lexer.line(),
                lexer.offset(),
                lexer.len(),
                token
            );
            if token == Token::Eof { break; }
        }
    }
}
