//! Assembler Messages
//
// This file is part of ras, an assembler.
// Copyright 2020 Jeffrey Sharp
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

use std::fmt::{self, Display, Formatter};
use crate::util::Location;

use crate::asm::Result;

// -----------------------------------------------------------------------------

/// Trait for types that log assembler messages.
pub trait Log {
    /// Logs the given message at `Normal` severity.  Returns `Ok(())`.
    fn log<M: Display + ?Sized>(&mut self, msg: &M) -> Result;

    /// Logs the given message at `Warning` severity.  Returns `Ok(())`.
    #[inline]
    fn log_warning<M: Display + ?Sized>(&mut self, msg: &M) -> Result {
        self.log(msg)
    }

    /// Logs the given message at `Error` severity.  Returns `Ok(())`.
    #[inline]
    fn log_error<M: Display + ?Sized>(&mut self, msg: &M) -> Result {
        self.log(msg)
    }

    /// Logs the given message at `Fatal` severity.  Returns `Err(())`.
    #[inline]
    fn log_fatal<M: Display + ?Sized>(&mut self, msg: &M) -> Result {
        let _ = self.log_error(msg);
        Err(())
    }
}

// -----------------------------------------------------------------------------

/// Trait for types that represent assembler messages.
pub trait Message: Display {
    /// Returns the severity of the message.
    #[inline]
    fn severity(&self) -> Severity {
        Severity::Error
    }

    /// Sends the message to the given log.
    #[inline]
    fn tell<L: Log>(&self, log: &mut L) -> Result {
        self.severity().dispatch(self, log)
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

impl Severity {
    /// Logs the given message, dispatching to the [`Log`] method appropriate
    /// for the message's severity.
    #[inline]
    fn dispatch<M, L>(self, msg: &M, log: &mut L) -> Result
    where
        M: Message + ?Sized,
        L: Log     + ?Sized,
    {
        use Severity::*;
        match self {
            Normal  => log.log(msg),
            Warning => log.log_warning(msg),
            Error   => log.log_error(msg),
            Fatal   => log.log_fatal(msg),
        }
    }
}

// Display is used when a Severity is printed in an assembler message.
impl Display for Severity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Severity::*;
        write!(f, "{}", match *self {
            Normal  => "",
            Warning => "warning: ",
            Error   => "error: ",
            Fatal   => "fatal: ",
        })
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct ReadError<'a>(
    pub &'a str,            // path
    pub &'a std::io::Error  // error
);

impl Message for ReadError<'_> { }

impl Display for ReadError<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let ReadError(path, err) = *self;
        write!(f, "{}{}: {}reading {}: {}", "ras", "", self.severity(), path, err)
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct WriteError<'a>(
    pub &'a str,            // path
    pub &'a std::io::Error  // error
);

impl Message for WriteError<'_> { }

impl Display for WriteError<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let WriteError(path, err) = *self;
        write!(f, "{}{}: {}writing {}: {}", "ras", "", self.severity(), path, err)
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct SyntaxError<'a>(
    pub &'a str,            // path
    pub Location            // line/column location
);

impl Message for SyntaxError<'_> { }

impl Display for SyntaxError<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let SyntaxError(path, loc) = *self;
        write!(f, "{}{}: {}syntax error", path, loc, self.severity())
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct General<M: Message> {
    pub msg: M,
}

impl<M: Message> Display for General<M> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "ras: {}{}", self.msg.severity(), self.msg)
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct Located<'a, M: Message> {
    pub path: &'a str,
    pub loc:  Location,
    pub msg:  M,
}

impl<M: Message> Display for Located<'_, M> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{}: {}{}", self.path, self.loc, self.msg.severity(), self.msg)
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    //use super::*;
}
