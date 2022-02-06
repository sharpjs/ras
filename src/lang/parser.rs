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
        self.parse_block(Eof).unwrap()
    }

    // Rules:
    //
    // If code receives a token, either as a method parameter, as returned from
    // a call, or via a &mut token parameter in a call, the lexer is positioned
    // at that token.  Otherwise, the lexer is positioned before the next token
    // and the method should get the next one as needed.  Document exceptions.
    //
    // BOF is the lexer state before its first `next` call.
    //
    // EOS is the `Eos` token for directives.
    // EOS is a label declarator token for labels.
    //
    // EOF is the `Eof` token.

    /// Attempts to parse a block with the given `end` token.
    /// Fails on unexpected EOF.
    ///
    /// Lexer positions:
    /// - On entry: at BOF or on block opening delimiter.
    /// - On exit:  at EOF or on block closing delimiter.
    fn parse_block(&mut self, end: Token) -> Result<Block, ()> {
        let mut stmts = vec![];

        'block: loop {
            match self.parse_stmt() {
                Ok(stmt) => {
                    // Accumulate statement
                    stmts.push(stmt);
                },
                Err(token) if token == end => {
                    // Complete block
                    break;
                },
                Err(RCurly) => {
                    // } not in {} block
                    eprintln!("error: unexpected '}}'");
                },
                Err(Eof) => {
                    // EOF not in top-level block
                    eprintln!("error: unexpected end of file");
                    return Err(());
                }
                Err(_) => {
                    // Other weirdness
                    eprintln!("error: expected statement");

                    // Recover
                    'recov: loop {
                        match self.lexer.next() {
                            t if t == end => break 'block,
                            Eos | Eof     => break 'recov,
                            _             => (),
                        }
                    }
                }
            }
        }

        Ok(Block { stmts, data: () })
    }

    /// Attempts to parse a statement.
    ///
    /// Lexer positions:
    /// - On entry:   before statement token.
    /// - On success: after statement, on EOS or EOF.
    /// - On failure: on unexpected token (returned).
    fn parse_stmt(&mut self) -> Result<Box<Stmt>, Token> {
        loop {
            match self.lexer.next() {
                Eos => {
                    // Ignore empty statement
                },
                Ident => {
                    // Parse as a label or directive
                    if let Ok(stmt) = self.parse_label_or_dir() {
                        return Ok(stmt)
                    }
                    // Assume `parse_label_or_dir` added errors and recovered
                    // to EOS; look again for a statement
                },
                token => {
                    // Report non-statement token
                    return Err(token);
                },
            }
        }
    }

    /// Attempts to parse a label or a directive.
    ///
    /// Lexer positions:
    /// - On entry: on [`Ident`].
    /// - On exit:  on EOS or EOF.
    fn parse_label_or_dir(&mut self) -> Result<Box<Stmt>, ()> {
        // Get label or directive name
        let name   = self.lexer.str();
        let pseudo = name.starts_with('.');
        let name   = self.session.names_mut().add(name);

        // Expect label declarator as EOS; otherwise parse as directive
        let scope = match self.lexer.next() {
            Colon  if pseudo => Scope::Local,
            Colon            => Scope::Private,
            Weak             => Scope::Weak,
            Public if pseudo => Scope::Hidden,
            Public           => Scope::Public,
            token            => return self.parse_dir(name, token),
        };

        Ok(Box::new(Stmt::Label(Label { name, scope, data: () })))
    }

    /// Attempts to parse a directive with the given `name`.
    ///
    /// Lexer positions:
    /// - On entry: after `name`, on given `token`.
    /// - On exit:  on EOS or EOF.
    fn parse_dir(&mut self, name: Name, mut token: Token) -> Result<Box<Stmt>, ()> {
        let mut args = vec![];

        // Parse arguments if present
        if !token.is_eos() {
            loop {
                // Parse argument
                match self.parse_expr(token) {
                    Ok((arg, t)) => {
                        args.push(arg);
                        token = t;
                    },
                    Err(t) => {
                        eprintln!("expected: argument");
                        return self.parse_dir_fail(t);
                    },
                }

                // Parse argument separator or end of statement
                match token {
                    Comma     => token = self.lexer.next(),
                    Eos | Eof => break,
                    _ => {
                        eprintln!("expected: comma, end of statement, or end of file");
                        return self.parse_dir_fail(token);
                    },
                }
            }
        }

        Ok(Box::new(Stmt::Op(Op { name, args, data: () })))
    }

    fn parse_dir_fail(&mut self, mut token: Token) -> Result<Box<Stmt>, ()> {
        // Recover
        while !token.is_eos() {
            token = self.lexer.next();
        }

        Err(())
    }

    /// Attempts to parse an expression.
    ///
    /// Lexer positions:
    /// - On entry:   at `token`, the first token of the expression.
    /// - On success: at the returned token, the first token after the expression.
    /// - On failure: at the returned token, the token that was unexpected.
    #[inline]
    fn parse_expr(&mut self, token: Token) -> Result<(Box<Expr>, Token), Token> {
        self.parse_expr_prec(token, 0)
    }

    /// Attempts to parse an expression with the given minimum precedence.
    ///
    /// This method is the postfix half of the precedence-climbing expression
    /// parser.
    ///
    /// Lexer positions:
    /// - On entry:   at `token`, the first token of the expression.
    /// - On success: at the returned token, the first token after the expression.
    /// - On failure: at the returned token, the token that was unexpected.
    fn parse_expr_prec(&mut self, token: Token, min_prec: u8)
        -> Result<(Box<Expr>, Token), Token>
    {
        use PostfixParse as P;

        let (mut expr, mut token) = self.parse_expr_prefix(token)?;

        loop {
            match postfix_parse_kind(token) {
                P::None => {
                    break;
                },
                P::Unary(op) => {
                    let (prec, _assoc) = unary_prec(op);
                    if prec < min_prec { break; }

                    token = self.lexer.next();
                    expr  = Box::new(Expr::Unary((), op, expr));
                },
                P::Binary(op) => {
                    let (prec, assoc) = binary_prec(op);
                    if prec < min_prec { break; }

                    token = self.lexer.next();
                    let (rhs, t) = self.parse_expr_prec(token, prec + assoc as u8)?;

                    token = t;
                    expr  = Box::new(Expr::Binary((), op, expr, rhs))
                },
            }
        }

        Ok((expr, token))
    }

    /// Attempts to parse an atomic, prefix, or circumfix expression.
    ///
    /// This method is the prefix half of the precedence-climbing expression
    /// parser.
    ///
    /// Lexer positions:
    /// - On entry:   at `token`, the first token of the expression.
    /// - On success: at the returned token, the first token after the expression.
    /// - On failure: at the returned token, the token that was unexpected.
    fn parse_expr_prefix(&mut self, token: Token)
        -> Result<(Box<Expr>, Token), Token>
    {
        use PrefixParse as P;

        match prefix_parse_kind(token) {
            P::Ident => Ok({
                let name = self.name();
                match self.lexer.next() {
                    Alias => {
                        let (prec, assoc) = ALIAS_PREC;
                        let        token  = self.lexer.next();
                        let (expr, token) = self.parse_expr_prec(token, prec + assoc as u8)?;
                        (Box::new(Expr::Alias((), name, expr)), token)
                    },
                    token => (Box::new(Expr::Ident((), name)), token)
                }
            }),
            P::Param => todo!(), // TODO: Process macro right here?
            P::Int => Ok((
                Box::new(Expr::Int((), self.lexer.int())),
                self.lexer.next()
            )),
            P::Float => Ok((
                Box::new(Expr::Float((), ())),
                self.lexer.next()
            )),
            P::Str => Ok((
                Box::new(Expr::Str((), self.lexer.str().to_string())),
                self.lexer.next()
            )),
            P::Char => Ok((
                Box::new(Expr::Char((), self.lexer.char())),
                self.lexer.next()
            )),
            P::Unary(op) => {
                let (prec, assoc) = unary_prec(op);
                let        token  = self.lexer.next();
                let (expr, token) = self.parse_expr_prec(token, prec + assoc as u8)?;
                Ok(( Box::new(Expr::Unary((), op, expr)), token ))
            },
            P::Group => {
                let       token  = self.lexer.next();
                let (lhs, token) = self.parse_expr(token)?;
                match token {
                    RParen => Ok((lhs, self.lexer.next())),
                    _      => Err({ eprintln!("expected: ')'"); token }),
                }
            },
            P::Deref => {
                let       token  = self.lexer.next();
                let (lhs, token) = self.parse_expr(token)?;
                match token {
                    RSquare => {
                        let (effect, token) = match self.lexer.next() {
                            LogNot => (true, self.lexer.next()),
                            token  => (false, token),
                        };
                        Ok(( Box::new(Expr::Deref((), lhs, effect)), token ))
                    },
                    _ => Err({ eprintln!("expected: ']'"); token }),
                }
            },
            P::Block => {
                let block = self.parse_block(RCurly);
                let token = self.lexer.next();
                match block {
                    Ok(block) => Ok((Box::new(Expr::Block(block)), token)),
                    _         => Err(token),
                }
            },
            P::None => Err({
                eprintln!("expected: expression");
                token
            }),
        }
    }

    fn name(&mut self) -> Name {
        self.session.names_mut().add(self.lexer.str())
    }
}

// ----------------------------------------------------------------------------

/// Expression parsing strategies selected by leading token.
///
/// These cases include atomic, prefix, and circumfix expressions.
#[derive(Clone, Copy, Debug)]
enum PrefixParse {
    /// Do not parse.
    None,

    /// Parse as an identifier atom.
    Ident,

    /// Parse as a macro parameter atom.
    Param,

    /// Parse as an integer literal atom.
    Int,

    /// Parse as a floating-point number literal atom.
    Float,

    /// Parse as a string literal atom.
    Str,

    /// Parse as a character literal atom.
    Char,

    /// Parse as a unary operator expression.
    Unary(UnOp),

    /// Parse as a grouping expression.
    Group,

    /// Parse as a dereference expression.
    Deref,

    /// Parse as a statement block expression.
    Block,
}

/// Returns the expression parsing strategy for the given leading `token`.
const fn prefix_parse_kind(token: Token) -> PrefixParse {
    use PrefixParse as P;
    match token {
        Ident    => P::Ident,
        Param    => P::Param,
        Int      => P::Int,
        Float    => P::Float,
        Str      => P::Str,
        Char     => P::Char,

        BitNot   => P::Unary(UnOp::BitNot),
        LogNot   => P::Unary(UnOp::LogNot),
        Inc      => P::Unary(UnOp::PreInc),
        Dec      => P::Unary(UnOp::PreDec),
        Mod      => P::Unary(UnOp::UnsignedH),
        Add      => P::Unary(UnOp::SignedH),
        Sub      => P::Unary(UnOp::Neg),

        LParen   => P::Group,
        LSquare  => P::Deref,
        LCurly   => P::Block,

        Unsigned => P::Unary(UnOp::UnsignedL),
        Signed   => P::Unary(UnOp::SignedL),

        _        => P::None,
    }
}

// ----------------------------------------------------------------------------

/// Expression parsing strategies determined by trailing token.
///
/// These cases include postfix and infix exprssions.
#[derive(Clone, Copy, Debug)]
enum PostfixParse {
    /// Do not parse.
    None,

    /// Parse as a unary operator expression.
    Unary(UnOp),

    /// Parse as a binary operator expression.
    Binary(BinOp),
}

/// Returns the expression parsing strategy for the given trailing `token`.
const fn postfix_parse_kind(token: Token) -> PostfixParse {
    use PostfixParse::*;
    match token {
        //Alias       => handled as special case in PrefixParse::Ident

        Inc           => Unary(UnOp::PostInc),
        Dec           => Unary(UnOp::PostDec),

        //LParen      => Call,
        //LSquare     => Index,

        Mul           => Binary(BinOp::Mul),
        Div           => Binary(BinOp::Div),
        Mod           => Binary(BinOp::Mod),

        Add           => Binary(BinOp::Add),
        Sub           => Binary(BinOp::Sub),

        Shl           => Binary(BinOp::Shl),
        Shr           => Binary(BinOp::Shr),

        BitAnd        => Binary(BinOp::BitAnd),
        BitXor        => Binary(BinOp::BitXor),
        BitOr         => Binary(BinOp::BitOr),

        Eq            => Binary(BinOp::Eq),
        NotEq         => Binary(BinOp::NotEq),
        Less          => Binary(BinOp::Less),
        More          => Binary(BinOp::More),
        LessEq        => Binary(BinOp::LessEq),
        MoreEq        => Binary(BinOp::MoreEq),

        LogAnd        => Binary(BinOp::LogAnd),
        LogXor        => Binary(BinOp::LogXor),
        LogOr         => Binary(BinOp::LogOr),

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

        BitNot        => Binary(BinOp::Range),
        Colon         => Binary(BinOp::Join),

        _             => None,
    }
}

// ----------------------------------------------------------------------------

/// Operator associativity kinds.
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum Assoc {
    /// Left-associative.
    Left,

    /// Right-associative.
    Right
}

/// Precedence and associativity of alias operator.
const ALIAS_PREC: (u8, Assoc) = (16, Assoc::Right);

/// Returns the precedence and associativity of the given unary operator.
const fn unary_prec(op: UnOp) -> (u8, Assoc) {
    use UnOp::*;
    use Assoc::*;
    match op {                                      // prec assoc
        PostInc | PostDec                           => (15, Left ),

        PreInc  | PreDec                            |
        BitNot  | LogNot | Neg                      |
        SignedH | UnsignedH                         => (14, Right),

        SignedL | UnsignedL                         => ( 0, Right),
    }
}

/// Returns the precedence and associativity of the given binary operator.
const fn binary_prec(op: BinOp) -> (u8, Assoc) {
    use BinOp::*;
    use Assoc::*;
    match op {                                      // prec assoc
        Mul | Div | Mod                             => (13, Left ),
        Add | Sub                                   => (12, Left ),
        Shl | Shr                                   => (11, Left ),
        BitAnd                                      => (10, Left ),
        BitXor                                      => ( 9, Left ),
        BitOr                                       => ( 8, Left ),
        Eq | NotEq | Less | More | LessEq | MoreEq  => ( 7, Left ),
        LogAnd                                      => ( 6, Left ),
        LogXor                                      => ( 5, Left ),
        LogOr                                       => ( 4, Left ),

        Assign                                      |
        MulAssign    | DivAssign    | ModAssign     |
        AddAssign    | SubAssign                    |
        ShlAssign    | ShrAssign                    |
        BitAndAssign | BitXorAssign | BitOrAssign   |
        LogAndAssign | LogXorAssign | LogOrAssign   => ( 3, Right),

        Range                                       => ( 2, Right),
        Join                                        => ( 1, Right),
    }
}
