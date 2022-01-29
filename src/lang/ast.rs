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

//! Abstract syntax trees.

use std::fmt::{self, Display, Formatter};
use crate::name::{Name, NameTable};

/// Module.
///
/// ```text
/// module = EOS* stmt*
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Module<T = ()> {
    /// Statements in the module.
    pub stmts: Vec<Box<Stmt<T>>>,

    /// Additional data.
    pub data: T,
}

/// Statement.
///
/// ```text
/// stmt = label | directive
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stmt<T = ()> {
    /// Label statement.
    Label(Label<T>),

    /// Directive statement.
    Directive(Directive<T>),
}

/// Label.
///
/// ```text
/// label = IDENT (":" | ":?" | "::") EOS*
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Label<T = ()> {
    /// Name.
    pub name: Name,

    /// Scope.
    pub scope: Scope,

    /// Additional data.
    pub data: T,
}

/// Symbol scopes.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Scope {
    /// Valid between non-local labels in the same source file.  Not present in
    /// the object file.
    ///
    /// Lexical form: `.foo:`
    Local,

    /// Valid within the entire source file.  Not present in the object file.
    ///
    /// Lexical form: `.foo::`
    Hidden,

    /// Valid within the entire source file.  Present in the object file but
    /// not exported to other objects.
    ///
    /// Lexical form: `foo:`
    Private,

    /// Valid within all source files.  Present in the object file and exported
    /// to other objects.  Overridable by a symbol with [`Scope::Public`]
    /// scope.
    ///
    /// Lexical form: `foo:?`
    Weak,

    /// Valid within all source files.  Present in the object file and exported
    /// to other objects.  Not overridable.
    ///
    /// Lexical form: `foo::`
    Public,
}

/// Directive.
///
/// ```text
/// directive = IDENT arguments?
/// arguments = term ( "," term )*
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Directive<T = ()> {
    /// Name.
    pub name: Name,

    /// Additional data.
    pub data: T,
}

/// Expression.
///
/// ```text
/// TODO: pseudo-BNF
/// ```
#[derive(Clone, PartialEq, /*Eq,*/ Debug)]
pub enum Expr<T = ()> {
    // Atoms

    /// Identifier.
    Ident(T, Name),

    /// Integer literal.
    Int(T, u64),

    /// Floating-point literal.
    Float(T, f64),

    /// String literal.
    Str(T, String),

    /// Character literal.
    Char(T, char),

    // Operators
    // one variant per tuple type

    // Statement block.
    Block(T, Vec<Box<Stmt<T>>>),

    /// Dereference expression: `[expr]` `[expr]!`
    Deref(T, Box<Expr<T>>, bool),

    /// Compound name expression: `name:name`.
    Join(T, Name, Name),

    /// Alias expression: `name@expr`.
    Alias(T, Name, Box<Expr<T>>),

    /// Unary operation on a subexpression.
    Unary(T, UnOp, Box<Expr<T>>),

    /// Binary operation on subexpressions.
    Binary(T, BinOp, Box<Expr<T>>, Box<Expr<T>>),
}

// ----------------------------------------------------------------------------

/// Unary operators on expressions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnOp {
// Suffix unary, precedence level 13

    /// `x++` - post-increment operator.
    PostInc,

    /// `x--` - post-increment operator.
    PostDec,

// Prefix unary, precedence level 12

    /// `~x` - bitwise NOT operator.
    BitNot,

    /// `!x` - logical NOT operator.
    LogNot,

    /// `-x` - arithmetic negation operator.
    Neg,

    /// `+x` - explicit-signed operator.
    SignedH,

    /// `%x` - explicit-unsigned operator.
    UnsignedH,

    /// `++x` - pre-increment operator.
    PreInc,

    /// `--x` - pre-decrement operator.
    PreDec,

// Prefix unary, precedence level 0

    /// `+:` - implicit-signed operator.
    SignedL,

    /// `%:` - implicit-unsigned operator.
    UnsignedL,
}

// ----------------------------------------------------------------------------

/// Binary operators on expressions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
// Multiplicative, precedence level 11

    /// `*` - multiplication operator.
    Mul,

    /// `/` - division operator.
    Div,

    /// `%` - modulo operator.
    Mod,

// Additive, precedence level 10

    /// `+` - addition operator.
    Add,

    /// `-` - subtraction operator.
    Sub,

// Shift, precedence level 9

    /// `<<` - left shift operator.
    Shl,

    /// `>>` - right shift operator.
    Shr,

// Bitwise AND/XOR/OR, precedence levels 6-8

    /// `&` - bitwise AND operator.
    BitAnd,

    /// `^` - bitwise XOR operator.
    BitXor,

    /// `|` - bitwise OR operator.
    BitOr,

// Comparison, precedence level 5

    /// `==` - equality operator.
    Eq,

    /// `!=` - inequality operator.
    NotEq,

    /// `<` - less-than operator.
    Less,

    /// `>` - greater-than operator.
    More,

    /// `<=` - less-than-or-equal-to operator.
    LessEq,

    /// `>=` - greater-than-or-equal-to operator.
    MoreEq,

// Logical AND/XOR/OR, precedence level 2-4

    /// `&&` - logical AND operator.
    LogAnd,

    /// `^^` - logical XOR operator.
    LogXor,

    /// `||` - logical OR operator.
    LogOr,

// Assignment, precedence level 1

    /// `=` - assignment operator.
    Assign,

    /// `*=` - compound multiplication-assignment operator.
    MulAssign,

    /// `/=` - compound division-assignment operator.
    DivAssign,

    /// `&=` - compound modulo-assignment operator.
    ModAssign,

    /// `+=` - compound addition-assignment operator.
    AddAssign,

    /// `-=` - compound subtraction-assignment operator.
    SubAssign,

    /// `<<=` - compound left-shift-assignment operator.
    ShlAssign,

    /// `>>=` - compound right-shift-assignment operator.
    ShrAssign,

    /// `&=` - compound bitwise-AND-assignment operator.
    BitAndAssign,

    /// `^=` - compound bitwise-XOR-assignment operator.
    BitXorAssign,

    /// `|=` - compound bitwise-OR-assignment operator.
    BitOrAssign,

    /// `&&=` - compound logical-AND-assignment operator.
    LogAndAssign,

    /// `^^=` - compound logical-XOR-assignment operator.
    LogXorAssign,

    /// `||=` - compound logical-OR-assignment operator.
    LogOrAssign,
}

// ----------------------------------------------------------------------------

/// Node wrapper to facilitate [`Display`] implementation.
#[derive(Clone, Copy, Debug)]
struct ForDisplay<'a, T> {
    node:    &'a T,
    names:   &'a NameTable,
    nesting: Nesting<'a>,
}

#[derive(Clone, Copy, Debug)]
enum Nesting<'a> {
    Root,
    Child { more: bool, parent: &'a Self }
}

 // TODO: Figure out a better way or name
#[derive(Clone, Copy, Debug)]
struct Nesting2<'a> (Nesting<'a>);

impl Module {
    /// Returns a wrapper over the node that implements [`Display`].
    pub fn for_display<'a>(&'a self, names: &'a NameTable) -> impl Display + 'a {
        ForDisplay { node: self, names, nesting: Nesting::Root }
    }
}

impl<T> ForDisplay<'_, T> {
    fn drill<'a, U>(&'a self, node: &'a U) -> ForDisplay<'a, U> {
        ForDisplay { node, names: self.names, nesting: self.nesting }
    }

    fn child<'a, U>(&'a self, node: &'a U, more: bool) -> ForDisplay<'a, U> {
        let nesting = Nesting::Child { more, parent: &self.nesting };
        ForDisplay { node, names: self.names, nesting }
    }
}

impl Display for Nesting2<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Nesting::*;
        match self.0 {
            Root                          => Ok(()),
            Child { more: false, parent } => write!(f, "{}╰─", parent),
            Child { more: true,  parent } => write!(f, "{}├─", parent),
        }
    }
}

impl Display for Nesting<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Nesting::*;
        match *self {
            Root                          => Ok(()),
            Child { more: false, parent } => write!(f, "{}  ", parent),
            Child { more: true,  parent } => write!(f, "{}│ ", parent),
        }
    }
}

impl<T> Display for ForDisplay<'_, Module<T>> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "{}[module]", Nesting2(self.nesting))?;
        if let Some((x, xs)) = self.node.stmts.split_last() {
            xs.iter().try_for_each(|stmt| self.child(&**stmt, true).fmt(f))?;
            self.child(&**x, false).fmt(f)
        } else {
            Ok(())
        }
    }
}

impl<T> Display for ForDisplay<'_, Stmt<T>> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Stmt::*;
        match *self.node {
            Label     (ref l) => self.drill(l).fmt(f),
            Directive (ref d) => self.drill(d).fmt(f),
        }
    }
}

impl<T> Display for ForDisplay<'_, Label<T>> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f,
            "{}[label] {} ({:?})",
            Nesting2(self.nesting),
            self.names.get(self.node.name),
            self.node.scope
        )
    }
}

impl<T> Display for ForDisplay<'_, Directive<T>> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f,
            "{}[op] {}",
            Nesting2(self.nesting),
            self.names.get(self.node.name)
        )
    }
}

impl<T> Display for ForDisplay<'_, Expr<T>> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Expr::*;
        match *self.node {
            Ident(_, name) => {
                writeln!(f, "{}[ident] {}", Nesting2(self.nesting), self.names.get(name))
            },

            Int  (_, _)    => todo!(),
            Float(_, _)    => todo!(),
            Str  (_, _)    => todo!(),
            Char (_, _)    => todo!(),
            Block(_, _)    => todo!(),
            Deref(_, _, _) => todo!(),
            Join (_, _, _) => todo!(),
            Alias(_, _, _) => todo!(),

            Unary(_, op, ref expr) => {
                writeln!(f, "{}[unary] {:?}", Nesting2(self.nesting), op)?;
                self.child(&**expr, false).fmt(f)
            },
            Binary(_, op, ref lhs, ref rhs) => {
                writeln!(f, "{}[binary] {:?}", Nesting2(self.nesting), op)?;
                self.child(&**lhs, true ).fmt(f)?;
                self.child(&**rhs, false).fmt(f)
            },
        }
    }
}
