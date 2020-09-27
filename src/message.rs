// This file is part of ras, an assembler.
// Copyright (C) 2020 Jeffrey Sharp
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

//! Assembler Messages

use std::fmt::{self, Arguments, Display, Formatter};
use crate::util::Location;

// -----------------------------------------------------------------------------

/// Trait for assembler message types.
pub trait Message: Display {
    /// Returns the origin (e.g. path, line, and column) of the message.
    #[inline]
    fn origin(&self) -> Origin { Origin::General }

    /// Returns the severity level of the message.
    #[inline]
    fn severity(&self) -> Severity { Severity::Normal }
}

impl Message for str           {}
impl Message for String        {}
impl Message for Arguments<'_> {}

impl<T> Message for &T where T: Message + ?Sized {
    #[inline]
    fn origin(&self) -> Origin { (*self).origin() }

    #[inline]
    fn severity(&self) -> Severity { (*self).severity() }
}

// -----------------------------------------------------------------------------

/// Assembler message origins.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Origin<'a> {
    /// The message originates from the assembler itself.
    General,

    /// The message originates from a source code file.
    File {
        /// Path of the source code file.
        path: &'a str,

        /// Line-and-column location within the source code file.
        loc: Location
    },
}

impl Display for Origin<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Origin::General            => write!(f, "{}", crate::PROGRAM_NAME),
            Origin::File { path, loc } => write!(f, "{}:{}", path, loc),
        }
    }
}

// -----------------------------------------------------------------------------

/// Wrapper type that adds file origin information to an assembler message.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FileMessage<'a, M: Message> {
    /// The assembler message.
    msg: M,

    /// Path of the source code file.
    path: &'a str,

    /// Line-and-column location within the source code file.
    loc: Location,
}

impl<'a, M: Message> Message for FileMessage<'a, M> {
    #[inline]
    fn origin(&self) -> Origin {
        Origin::File { path: self.path, loc: self.loc }
    }

    #[inline]
    fn severity(&self) -> Severity {
        self.msg.severity()
    }
}

impl<'a, M: Message> Display for FileMessage<'a, M> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.msg.fmt(f)
    }
}

// -----------------------------------------------------------------------------

/// Message severity levels.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Severity {
    /// For informational messages.
    Normal,

    /// For potential problems that do not prevent complete assembly.
    /// Assembly continues, and the assembler will produce output.
    Warning,

    /// For problems that prevent complete assembly.
    /// Assembly might continue, but the assembler will not produce output.
    Error,

    /// For severe, unrecoverable problems.
    /// The assembler terminates immediately and does not produce output.
    Fatal,
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(match *self {
            Severity::Normal  => "",
            Severity::Warning => "warning: ",
            Severity::Error   => "error: ",
            Severity::Fatal   => "fatal: ",
        })
    }
}

// -----------------------------------------------------------------------------

/// Wrapper type that gives warning severity to an assembler message.
#[derive(Copy, Clone, Debug)]
pub struct Warning<T: Message>(T);

impl<T: Message> Message for Warning<T> {
    #[inline]
    fn origin(&self) -> Origin {
        self.0.origin()
    }

    #[inline]
    fn severity(&self) -> Severity {
        Severity::Warning
    }
}

impl<T: Message> Display for Warning<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

// -----------------------------------------------------------------------------

/// Wrapper type that gives error severity to an assembler message.
#[derive(Copy, Clone, Debug)]
pub struct Error<T: Message>(T);

impl<T: Message> Message for Error<T> {
    #[inline]
    fn origin(&self) -> Origin {
        self.0.origin()
    }

    #[inline]
    fn severity(&self) -> Severity {
        Severity::Error
    }
}

impl<T: Message> Display for Error<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

// -----------------------------------------------------------------------------

/// Wrapper type that gives fatal severity to an assembler message.
#[derive(Copy, Clone, Debug)]
pub struct Fatal<T: Message>(T);

impl<T: Message> Message for Fatal<T> {
    #[inline]
    fn origin(&self) -> Origin {
        self.0.origin()
    }

    #[inline]
    fn severity(&self) -> Severity {
        Severity::Fatal
    }
}

impl<T: Message> Display for Fatal<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

// -----------------------------------------------------------------------------

pub fn file_not_found_error(path: &str) -> impl Message + '_ {
    FileMessage {
        msg: Error("file not found"),
        path: path,
        loc: Location::UNKNOWN,
    }
}
