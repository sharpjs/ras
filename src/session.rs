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

//! Assembly session.

use crate::lang::lexer::{Lex, Lexer, Token};
use crate::lang::parser::Parser;
use crate::name::NameTable;

// ----------------------------------------------------------------------------

/// Assembler session.
#[derive(Debug)]
pub struct Session {
    names: NameTable,
}

impl Session {
    /// Creates a new [`Session`].
    pub fn new() -> Self {
        Self {
            names: NameTable::new(),
        }
    }

    pub fn names(&self) -> &NameTable {
        &self.names
    }

    pub fn names_mut(&mut self) -> &mut NameTable {
        &mut self.names
    }

    pub fn print_tokens(&mut self, path: &str, content: &str) {
        println!("[{}:tokens]", path);

        //        0         1         2         3         4         5         6         7         8
        //        0 2 4 6 8 0 2 4 6 8 0 2 4 6 8 0 2 4 6 8 0 2 4 6 8 0 2 4 6 8 0 2 4 6 8 0 2 4 6 8 0
        println!("╭──────┬────────┬────────┬───────┬──────────────────────╮");
        println!("│ LINE │ OFFSET │ LENGTH │ TYPE  │ VALUE                │");
        println!("╞══════╪════════╪════════╪═══════╪══════════════════════╡");

        let mut lexer = Lexer::new(content.bytes());

        loop {
            let token = lexer.next();
            println!(
                "│ {:4} │ {:6} │ {:6} │ {:5} │ {:<20.20} │",
                lexer.line(),
                lexer.range().start,
                lexer.range().len(),
                token,
                lexer.value(token)
            );
            if token == Token::Eof { break; }
        }
        println!("╰──────┴────────┴────────┴───────┴──────────────────────╯");
    }

    pub fn print_ast(&mut self, path: &str, content: &str) {
        println!("[{}:ast]", path);

        let     lexer = Lexer::new(content.bytes());
        let mut parser = Parser::new(lexer, self);

        let ast = parser.parse();

        println!("{}", ast.for_display(self.names()))
    }
}
