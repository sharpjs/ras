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

// ----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct ForDisplay<'a, T> {
    node:  &'a T,
    names: &'a NameTable,
}

impl Module {
    pub fn for_display<'a>(&'a self, names: &'a NameTable) -> impl Display + 'a {
        ForDisplay { node: self, names }
    }
}

impl<T> ForDisplay<'_, T> {
    fn drill<'a, U>(&'a self, other: &'a U) -> ForDisplay<'a, U> {
        ForDisplay { node: other, names: self.names }
    }
}

impl<T> Display for ForDisplay<'_, Module<T>> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.node.stmts.iter().try_for_each(|stmt| self.drill(&**stmt).fmt(f))
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
        writeln!(f, "Label {} {:?}", self.names.get(self.node.name), self.node.scope)
    }
}

impl<T> Display for ForDisplay<'_, Directive<T>> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "Op {}", self.names.get(self.node.name))
    }
}
