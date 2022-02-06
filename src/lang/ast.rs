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
use colored::*;
use crate::name::{Name, NameTable};

/// Block of statements.
///
/// ```text
/// block = EOS* stmt*
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Block<T = ()> {
    /// Statements in the block.
    pub stmts: Vec<Box<Stmt<T>>>,

    /// Additional data.
    pub data: T,
}

/// Statement.
///
/// ```text
/// stmt = label | op
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stmt<T = ()> {
    /// Label.
    Label(Label<T>),

    /// Operation directive.
    Op(Op<T>),
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

/// Operation directive.
///
/// ```text
/// op   = IDENT args?
/// args = term ( "," term )*
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Op<T = ()> {
    /// Name.
    pub name: Name,

    /// Arguments.
    pub args: Vec<Box<Expr<T>>>,

    /// Additional data.
    pub data: T,
}

/// Expression.
///
/// ```text
/// TODO: pseudo-BNF
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expr<T = ()> {
    // Atoms

    /// Identifier.
    Ident(T, Name),

    /// Integer literal.
    Int(T, u64),

    /// Floating-point literal.
    Float(T, ()),

    /// String literal.
    Str(T, String),

    /// Character literal.
    Char(T, char),

    // Operators
    // one variant per tuple type

    // Statement block.
    Block(Block),

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
struct ForDisplay<'a, T: ?Sized> {
    node:    &'a T,
    names:   &'a NameTable,
    nesting: Nesting<'a>,
}

#[derive(Clone, Copy, Debug)]
struct DisplayNode0<'a> {
    kind:    &'a str,
    nesting: Nesting<'a>,
}

#[derive(Clone, Copy, Debug)]
struct DisplayNode1<'a, T0: Display> {
    kind:    &'a str,
    nesting: Nesting<'a>,
    data:    T0,
}

#[derive(Clone, Copy, Debug)]
struct DisplayNode2<'a, T0: Display, T1: Display> {
    kind:    &'a str,
    nesting: Nesting<'a>,
    data:    (T0, T1),
}

#[derive(Clone, Copy, Debug)]
enum Nesting<'a> {
    Root,
    Child { more: bool, parent: &'a Self }
}

#[derive(Clone, Copy, Debug)]
struct Indent<'a> (Nesting<'a>);

impl Block {
    /// Returns a wrapper over the node that implements [`Display`].
    pub fn for_display<'a>(&'a self, names: &'a NameTable) -> impl Display + 'a {
        ForDisplay { node: self, names, nesting: Nesting::Root }
    }
}

impl<T: ?Sized> ForDisplay<'_, T> {
    fn drill<'a, U: ?Sized>(&'a self, node: &'a U) -> ForDisplay<'a, U> {
        ForDisplay { node, names: self.names, nesting: self.nesting }
    }

    fn child<'a, U: ?Sized>(&'a self, node: &'a U, more: bool) -> ForDisplay<'a, U> {
        let nesting = Nesting::Child { more, parent: &self.nesting };
        ForDisplay { node, names: self.names, nesting }
    }

    fn node0<'a>(
        &'a self, kind: &'a str
    ) -> DisplayNode0 {
        DisplayNode0 { kind, nesting: self.nesting }
    }

    fn node1<'a, T0: Display>(
        &'a self, kind: &'a str, data: T0
    ) -> DisplayNode1<T0> {
        DisplayNode1 { kind, nesting: self.nesting, data }
    }

    fn node2<'a, T0: Display, T1: Display>(
        &'a self, kind: &'a str, data0: T0, data1: T1
    ) -> DisplayNode2<T0, T1> {
        DisplayNode2 { kind, nesting: self.nesting, data: (data0, data1) }
    }
}

impl Display for DisplayNode0<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f,
            "{}{}",
            Indent(self.nesting),
            self.kind.green()
        )
    }
}

impl<T0: Display> Display for DisplayNode1<'_, T0> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f,
            "{}{}({})",
            Indent(self.nesting),
            self.kind.green(),
            self.data
        )
    }
}

impl<T0: Display, T1: Display> Display for DisplayNode2<'_, T0, T1> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f,
            "{}{}({}, {})",
            Indent(self.nesting),
            self.kind.green(),
            self.data.0,
            self.data.1
        )
    }
}

impl Display for Nesting<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Nesting::*;
        match *self {
            Root => Ok(()),
            Child { more, parent } => {
                let text = if more { "│ " } else { "  " };
                write!(f, "{}{}", parent, text.white().dimmed())
            },
        }
    }
}

impl Display for Indent<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Nesting::*;
        match self.0 {
            Root => Ok(()),
            Child { more, parent } => {
                let text = if more { "├─" } else { "╰─" };
                write!(f, "{}{}", parent, text.white().dimmed())
            },
        }
    }
}

impl<T> Display for ForDisplay<'_, Block<T>> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.node0("Block").fmt(f)?;
        self.drill(&self.node.stmts).fmt(f)
    }
}

impl<T> Display for ForDisplay<'_, Stmt<T>> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Stmt::*;
        match *self.node {
            Label (ref l) => self.drill(l).fmt(f),
            Op    (ref d) => self.drill(d).fmt(f),
        }
    }
}

impl<T> Display for ForDisplay<'_, Label<T>> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.node2("Label",
            self.names.get(self.node.name),
            format!("{:?}", self.node.scope)
        ).fmt(f)
    }
}

impl<T> Display for ForDisplay<'_, Op<T>> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.node1("Dir", self.names.get(self.node.name)).fmt(f)?;
        self.drill(&self.node.args).fmt(f)
    }
}

impl<T> Display for ForDisplay<'_, Expr<T>> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Expr::*;
        match *self.node {
            Ident(_,     name ) => self.node1("Ident", self.names.get(name)).fmt(f),
            Int  (_,     val  ) => self.node1("Int",   val).fmt(f),
            Float(_,     _    ) => self.node1("Float", "?").fmt(f),
            Str  (_, ref val  ) => self.node1("Str",   val).fmt(f),
            Char (_,     val  ) => self.node1("Char",  val).fmt(f),
            Block(   ref block) => self.drill(block).fmt(f),

            Deref(_, ref expr, store) => {
                self.node1("Deref", if store {"!"} else {"_"}).fmt(f)?;
                self.child(&**expr, false).fmt(f)
            },

            Join (_, _, _)  => todo!(),
            Alias(_, _, _)  => todo!(),

            Unary(_, op, ref expr) => {
                self.node1("Unary", format!("{:?}", op)).fmt(f)?;
                self.child(&**expr, false).fmt(f)
            },
            Binary(_, op, ref lhs, ref rhs) => {
                self.node1("Binary", format!("{:?}", op)).fmt(f)?;
                self.child(&**lhs, true ).fmt(f)?;
                self.child(&**rhs, false).fmt(f)
            },
        }
    }
}

impl<'a, T> ForDisplay<'a, Vec<Box<T>>> {
    // NOTE: This is a pseudo-Display implementation, written to mesh well with
    // the code above but permitting a constraint on the `fmt` method.
    fn fmt<'b>(&'b self, f: &mut Formatter) -> fmt::Result
    where
        ForDisplay<'b, T>: Display
    {
        if let [nodes@.., last] = &self.node[..] {
            for node in nodes {
                self.child(&**node, true).fmt(f)?;
            }
            self.child(&**last, false).fmt(f)
        } else {
            Ok(())
        }
    }
}
