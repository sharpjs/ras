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

use crate::name::Name;
use crate::session::Session;

use super::ast::*;
use super::lexer::{Lex, Token, Token::*};

#[derive(Debug)]
pub struct Parser<'a, L: Lex> {
    lexer:   L,
    session: &'a mut Session,
}

impl<'a, L: Lex> Parser<'a, L> {
    /// Creates a new [`Parser`] for the given `lexer` and `session`.
    pub fn new(lexer: L, session: &'a mut Session) -> Self {
        Self { lexer, session }
    }

    /// Parses input completely, returning an abstract syntax tree.
    pub fn parse(&mut self) -> Block {
        self.parse_block()
    }

    /// Parses a block of statements.
    ///
    /// Assumes the lexer is at BOF or [`Eos`].
    fn parse_block(&mut self) -> Block {
        let mut stmts = vec![];

        while let Some(stmt) = self.parse_stmt() {
            stmts.push(stmt);
        }

        Block { stmts, data: () }
    }

    /// Attempts to parse a statement.  Returns [`None`] at EOF.
    ///
    /// Assumes the lexer is at BOF or [`Eos`].
    fn parse_stmt(&mut self) -> Option<Box<Stmt>> {
        loop {
            match self.lexer.next() {
                Eos => {
                    // Ignore empty statement
                },
                Eof => {
                    // Signal end-of-file
                    return None;
                },
                Ident => {
                    // Parse as a label or directive
                    if let stmt@Some(_) = self.parse_label_or_dir() {
                        return stmt
                    }
                },
                _ => {
                    // TODO: Add syntax error.
                    eprintln!("error: expected: label or directive");
                },
            }
        }
    }

    /// Attempts to parse a label or a directive.  Returns [`None`] if input is
    /// unparseable.
    ///
    /// Assumes the lexer is at [`Ident`].
    fn parse_label_or_dir(&mut self) -> Option<Box<Stmt>> {
        // Get label or directive name
        let name   = self.lexer.str();
        let pseudo = name.starts_with('.');
        let name   = self.session.names_mut().add(name);

        // Expect label suffix; otherwise parse as directive
        let scope = match self.lexer.next() {
            Colon  if pseudo => Scope::Local,
            Colon            => Scope::Private,
            Weak             => Scope::Weak,
            Public if pseudo => Scope::Hidden,
            Public           => Scope::Public,
            token            => return self.parse_dir(name, token),
        };

        Some(Box::new(Stmt::Label(Label { name, scope, data: () })))
    }

    /// Attempts to parse a directive with the given `name`.  Returns [`None`]
    /// if input is unparsable.
    ///
    /// Assumes the lexer is at the given `token`.
    fn parse_dir(&mut self, name: Name, mut token: Token) -> Option<Box<Stmt>> {
        let mut args = vec![];

        if !matches!(token, Eos | Eof) {
            loop {
                // Parse argument
                match self.parse_expr(&mut token) {
                    Some(arg) => {
                        args.push(arg);
                    },
                    _ => {
                        // TODO: Syntax error
                        eprintln!("expected: argument");
                        return None;
                    },
                }

                // Parse argument separator or end of statement
                match token {
                    Comma => {
                        token = self.lexer.next();
                    },
                    Eos | Eof => {
                        break;
                    },
                    _ => {
                        // TODO: Syntax error
                        eprintln!("expected: comma, end of statement, or end of file");
                        return None;
                    },
                }
            }
        }

        Some(Box::new(Stmt::Op(Op { name, args, data: () })))
    }

    fn parse_expr(&mut self, token: &mut Token) -> Option<Box<Expr>> {
        self.parse_expr_prec(token, 0)
    }

    fn parse_expr_prec(&mut self, token: &mut Token, min_prec: u8) -> Option<Box<Expr>> {
        use PostfixParse as P;

        let mut lhs = self.parse_primary(token)?;
        *token = self.lexer.next();

        loop {
            match postfix_parse_kind(*token) {
                P::None => {
                    break;
                },
                P::Unary(op) => {
                    // Unary postfix operation on expression
                    let (prec, _assoc) = unary_prec(op);
                    if prec < min_prec { break; }

                    *token = self.lexer.next();

                    lhs = Box::new(Expr::Unary((), op, lhs));
                },
                P::Binary(op) => {
                    // Binary operation on expressions
                    let (prec, assoc) = binary_prec(op);
                    if prec < min_prec { break; }

                    *token = self.lexer.next();

                    let rhs = self.parse_expr_prec(token, prec + assoc as u8)?;
                    lhs = Box::new(Expr::Binary((), op, lhs, rhs))
                },
            }
        }

        Some(lhs)
    }

    fn parse_primary(&mut self, token: &mut Token) -> Option<Box<Expr>> {
        use PrefixParse as P;

        match prefix_parse_kind(*token) {
            P::Ident => {
                Some(Box::new(Expr::Ident((), self.name())))
            },
            P::Int => {
                Some(Box::new(Expr::Int((), self.lexer.int())))
            },
            P::Float => {
                Some(Box::new(Expr::Float((), ())))
            },
            P::Str => {
                Some(Box::new(Expr::Str((), self.lexer.str().to_string())))
            },
            P::Char => {
                Some(Box::new(Expr::Char((), 'x')))
            },
            P::Group => {
                let lhs = self.parse_expr_prec(token, 0)?;
                match *token {
                    RParen => {
                        *token = self.lexer.next();
                        Some(lhs)
                    },
                    _ => {
                        eprintln!("expected: ')'");
                        None
                    }
                }
            },
            P::Deref => {
                let lhs = self.parse_expr_prec(token, 0)?;
                match *token {
                    RSquare => {
                        *token = self.lexer.next();
                        let effect = *token == LogNot;
                        if effect { *token = self.lexer.next(); }
                        Some(Box::new(Expr::Deref((), lhs, effect)))
                    },
                    _ => {
                        eprintln!("expected: ']'");
                        None
                    }
                }
            },
            P::Block => {
                todo!()
                //let block = self.parse_block(); // TODO: what about token here?
                //match *token {
                //    RCurly => {
                //        *token = self.lexer.next();
                //        Some(Box::new(Expr::Block(block)))
                //    },
                //    _ => {
                //        eprintln!("expected: '}}'");
                //        None
                //    }
                //}
            },
            _ => {
                // TODO: Add syntax error.
                eprintln!("expected: expression");
                None
            },
        }
    }

    fn name(&mut self) -> Name {
        self.session.names_mut().add(self.lexer.str())
    }
}

#[derive(Clone, Copy, Debug)]
enum PrefixParse {
    None,
    Ident,
    Param,
    Int,
    Float,
    Str,
    Char,
    Unary(UnOp),
    Deref,
    Group,
    Block,
}

fn prefix_parse_kind(token: Token) -> PrefixParse {
    use PrefixParse as P;
    match token {
        Ident   => P::Ident,
        Param   => todo!(),
        Int     => todo!(),
        Float   => todo!(),
        Str     => todo!(),
        Char    => todo!(),
        BitNot  => todo!(),
        LogNot  => todo!(),
        Inc     => todo!(),
        Dec     => todo!(),
        Mod     => todo!(),
        Add     => todo!(),
        Sub     => todo!(),
        LCurly  => todo!(),
        LParen  => todo!(),
        LSquare => todo!(),
        Eos     => todo!(),
        Eof     => todo!(),
        _       => P::None,
    }
}

#[derive(Clone, Copy, Debug)]
enum PostfixParse {
    None,
    Unary(UnOp),
    Binary(BinOp),
}

fn postfix_parse_kind(token: Token) -> PostfixParse {
    use PostfixParse::*;
    match token {
        // Unary

        Inc => Unary(UnOp::PostInc),
        Dec => Unary(UnOp::PostDec),

        // Binary

    //  Alias   => AliasDef,
    //  Colon   => NameTuple,

    //  LParen  => Call,
    //  LSquare => Index,

        Mul     => Binary(BinOp::Mul),
        Div     => Binary(BinOp::Div),
        Mod     => Binary(BinOp::Mod),

        Add     => Binary(BinOp::Add),
        Sub     => Binary(BinOp::Sub),

        Shl     => Binary(BinOp::Shl),
        Shr     => Binary(BinOp::Shr),

        BitAnd  => Binary(BinOp::BitAnd),
        BitXor  => Binary(BinOp::BitXor),
        BitOr   => Binary(BinOp::BitOr),

        Eq      => Binary(BinOp::Eq),
        NotEq   => Binary(BinOp::NotEq),
        Less    => Binary(BinOp::Less),
        More    => Binary(BinOp::More),
        LessEq  => Binary(BinOp::LessEq),
        MoreEq  => Binary(BinOp::MoreEq),

        LogAnd  => Binary(BinOp::LogAnd),
        LogXor  => Binary(BinOp::LogXor),
        LogOr   => Binary(BinOp::LogOr),

        Assign        => Binary(BinOp::Assign),
        MulAssign     => Binary(BinOp::MulAssign),
        DivAssign     => Binary(BinOp::DivAssign),
        ModAssign     => Binary(BinOp::ModAssign),
        AddAssign     => Binary(BinOp::AddAssign),
        SubAssign     => Binary(BinOp::SubAssign),
        ShlAssign     => Binary(BinOp::ShlAssign),
        ShrAssign     => Binary(BinOp::ShrAssign),
        BitAndAssign  => Binary(BinOp::BitAndAssign),
        BitXorAssign  => Binary(BinOp::BitXorAssign),
        BitOrAssign   => Binary(BinOp::BitOrAssign),
        LogAndAssign  => Binary(BinOp::LogAndAssign),
        LogXorAssign  => Binary(BinOp::LogXorAssign),
        LogOrAssign   => Binary(BinOp::LogOrAssign),

        // Other
        _ => None,
    }
}

/// Operator associativity kinds.
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum Assoc {
    /// Left-associative.
    Left,

    /// Right-associative.
    Right
}

fn unary_prec(op: UnOp) -> (u8, Assoc) {
    use UnOp::*;
    use Assoc::*;
    match op {                  // prec assoc
        PostInc | PostDec       => (13, Left ),

        PreInc  | PreDec        |
        BitNot  | LogNot | Neg  |
        SignedH | UnsignedH     => (12, Right),

        SignedL | UnsignedL     => ( 0, Right),
    }
}

fn binary_prec(op: BinOp) -> (u8, Assoc) {
    use BinOp::*;
    use Assoc::*;
    match op {                                      // prec assoc
        Mul | Div | Mod                             => (11, Left ),
        Add | Sub                                   => (10, Left ),
        Shl | Shr                                   => ( 9, Left ),
        BitAnd                                      => ( 8, Left ),
        BitXor                                      => ( 7, Left ),
        BitOr                                       => ( 6, Left ),
        Eq | NotEq | Less | More | LessEq | MoreEq  => ( 5, Left ),
        LogAnd                                      => ( 4, Left ),
        LogXor                                      => ( 3, Left ),
        LogOr                                       => ( 2, Left ),

              Assign                                |
           MulAssign |    DivAssign |   ModAssign   |
           AddAssign |    SubAssign                 |
           ShlAssign |    ShrAssign                 |
        BitAndAssign | BitXorAssign | BitOrAssign   |
        LogAndAssign | LogXorAssign | LogOrAssign   => ( 1, Right),
    }
}
