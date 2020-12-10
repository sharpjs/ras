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

//! Assembler message types.

use std::fmt::{self, Display, Formatter};
use crate::util::Location;

use crate::asm::Result;

// ----------------------------------------------------------------------------

/// Trait for types that log assembler messages.
pub trait Log {
    /// Logs the given message at `Normal` severity.  Returns `Ok(())`.
    fn log<M: Display>(&mut self, msg: M) -> Result;

    /// Logs the given message at `Warning` severity.  Returns `Ok(())`.
    #[inline]
    fn log_warning<M: Display>(&mut self, msg: M) -> Result {
        self.log(msg)
    }

    /// Logs the given message at `Error` severity.  Returns `Ok(())`.
    #[inline]
    fn log_error<M: Display>(&mut self, msg: M) -> Result {
        self.log(msg)
    }

    /// Logs the given message at `Fatal` severity.  Returns `Err(())`.
    #[inline]
    fn log_fatal<M: Display>(&mut self, msg: M) -> Result {
        let _ = self.log_error(msg);
        Err(())
    }
}

// ----------------------------------------------------------------------------

/// Trait for types that represent assembler messages.
pub trait Message: Display + Sized {
    /// Returns the severity of the message.
    #[inline]
    fn severity(&self) -> Severity {
        Severity::Error
    }

    /// Augments the message with the given `path` and `loc` metadata
    /// identifying the path and textual location of the source code that
    /// caused the message.
    #[inline]
    fn at(self, path: &str, loc: Location) -> Located<Self> {
        Located { msg: self, path, loc }
    }

    /// Sends the message to the given log.
    #[inline]
    fn tell<L: Log>(self, log: &mut L) -> Result {
        self.severity().dispatch(Full(self), log)
    }

    /// Formats the message for logging using the given formatter.
    ///
    /// *Full* in this context means that this method will augment the message
    /// with available contextual metadata, such as source and severity.
    fn fmt_full(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "ras: {}{}", self.severity(), self)
    }
}

// ----------------------------------------------------------------------------

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
    /// Dispatches the given message to the [`Log`] method appropriate for the
    /// message's severity.
    #[inline]
    fn dispatch<M: Display, L: Log>(self, msg: M, log: &mut L) -> Result {
        use Severity::*;
        match self {
            Normal  => log.log(msg),
            Warning => log.log_warning(msg),
            Error   => log.log_error(msg),
            Fatal   => log.log_fatal(msg),
        }
    }
}

impl Display for Severity {
    /// Formats the message for output in an assember message.
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

// ----------------------------------------------------------------------------

/// Wrapper struct that causes a [`Message`] to format (via [`Display`]) as a
/// full assembler message appropriate for logging.
///
/// *Full* in this context means that [`Display.fmt`] will augment the inner
/// message with available contextual metadata, such as source and severity.
#[derive(Debug)]
pub struct Full<M: Message>(pub M);

impl<M: Message> Display for Full<M> {
    /// Forwards invocation to [`Message::fmt_full`].
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt_full(f)
    }
}

// ----------------------------------------------------------------------------

/// Represents an I/O error that occurred when reading input.
#[derive(Debug)]
pub struct ReadError<'a>(
    pub &'a str,            // path
    pub &'a std::io::Error  // error
);

impl Message for ReadError<'_> { }

impl Display for ReadError<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let ReadError(path, err) = *self;
        write!(f, "reading {}: {}", path, err)
    }
}

// ----------------------------------------------------------------------------

/// Represents an I/O error that occurred when writing output.
#[derive(Debug)]
pub struct WriteError<'a>(
    pub &'a str,            // path
    pub &'a std::io::Error  // error
);

impl Message for WriteError<'_> { }

impl Display for WriteError<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let WriteError(path, err) = *self;
        write!(f, "writing {}: {}", path, err)
    }
}

// ----------------------------------------------------------------------------

/// Represents an 'invalid character sequence' error.
#[derive(Debug)]
pub struct InvalidCharsError<'a>(pub &'a str);

impl Message for InvalidCharsError<'_> { }

impl Display for InvalidCharsError<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,
            "invalid character sequence '{}'",
            self.0.escape_default().to_string()
        )
    }
}

// ----------------------------------------------------------------------------

/// Represents a syntax error.
#[derive(Debug)]
pub struct SyntaxError;

impl Message for SyntaxError { }

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "syntax error")
    }
}

// ----------------------------------------------------------------------------

/// Wrapper struct that adds source path and textual location metadata to an
/// assembler message.
#[derive(Debug)]
pub struct Located<'a, M: Message> {
    /// Inner message.
    pub msg: M,

    /// Path of the source file that caused the message.
    pub path: &'a str,

    /// Textual location that caused the message.
    pub loc: Location,
}

impl<'a, M: Message> Located<'a, M> {
    /// Returns a new [`Located`] value with the same inner message but with
    /// the given `path` and `src` (textual location).
    ///
    /// This method is a specialization of [`Message::at`] that avoids
    /// double-wrapping.
    pub fn at(self, path: &'a str, loc: Location) -> Self {
        Self { path, loc, .. self }
    }
}

impl<M: Message> Message for Located<'_, M> {
    fn fmt_full(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{}: {}{}", self.path, self.loc, self.severity(), self)
    }
}

impl<M: Message> Display for Located<'_, M> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.msg.fmt(f)
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    //use super::*;
}
