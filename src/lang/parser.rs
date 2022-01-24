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

//! Parser.

use crate::session::Session;

use super::ast::*;
use super::lexer::{Lex, Token, Token::*};

#[derive(Debug)]
pub struct Parser<'a, L: Lex> {
    lexer:   L,
    session: &'a mut Session,
}

impl<'a, L: Lex> Parser<'a, L> {
    pub fn new(lexer: L, session: &'a mut Session) -> Self {
        Self { lexer, session }
    }

    pub fn parse(&mut self) -> Module {
        let mut stmts = vec![];

        while let Some(stmt) = self.parse_statement() {
            stmts.push(stmt);
        }

        Module { stmts, data: () }
    }

    fn parse_statement(&mut self) -> Option<Box<Stmt>> {
        loop {
            match self.lexer.next() {
                // Ignore empty statements
                Eos => (),

                // Detect end of file
                Eof => return None,

                // Parse a label or a directive
                Ident => if let stmt@Some(_) = self.parse_label_or_directive() {
                    return stmt
                },

                // Fail on another token
                _ => eprintln!("error: expected: label or directive"), // TODO: Syntax error
            }
        }
    }

    fn parse_label_or_directive(&mut self) -> Option<Box<Stmt>> {
        // Get label or directive name
        let name   = self.lexer.str();
        let pseudo = name.starts_with('.');
        let name   = self.session.names_mut().add(name);

        // Expect label suffix; otherwise parse as directive
        let scope = match self.lexer.next() {
            Colon if pseudo   => Scope::Local,
            Colon             => Scope::Private,
            Weak              => Scope::Weak,
            Public if pseudo  => Scope::Hidden,
            Public            => Scope::Public,
            token             => return self.parse_directive(token),
        };

        Some(Box::new(Stmt::Label(Label { name, scope, data: () })))
    }

    fn parse_directive(&self, token: Token) -> Option<Box<Stmt>> {
        eprintln!("skipping token for now: {}", token);
        None
    }
}
