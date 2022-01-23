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

use crate::name::Name;

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
    Directive, // TODO
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
