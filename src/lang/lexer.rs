// This file is part of ras, an assembler.
// Copyright (C) 2019 Jeffrey Sharp
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

// The lexer contains some optimizations discussed in this article:
// http://nothings.org/computer/lexing.html

use crate::lang::token::Token;
use crate::util::ConstDefault;

use self::Action::*;
use self::EqClass::*;
use self::State::*;
use self::TransitionId::*;

// ---------------------------------------------------------------------------- 

/// Character equivalence classes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
enum EqClass {
    // Basics
    Eof     =  0 * State::COUNT as u16,  // end-of-file (pseudo)
    Space   =  1 * State::COUNT as u16,  // space, tab
    Cr      =  2 * State::COUNT as u16,  // carriage return
    Lf      =  3 * State::COUNT as u16,  // line feed
    Id      =  4 * State::COUNT as u16,  // a-z, A-Z, _, all code points above U+007F
    // AaCcEeFf Bb Dd Hh Oo _
    Digit   =  5 * State::COUNT as u16,  // 0-9
    // Open/close pairs
    LParen  =  6 * State::COUNT as u16,  // (
    RParen  =  7 * State::COUNT as u16,  // )
    LSquare =  8 * State::COUNT as u16,  // [
    RSquare =  9 * State::COUNT as u16,  // ]
    LCurly  = 10 * State::COUNT as u16,  // {
    RCurly  = 11 * State::COUNT as u16,  // }
    // Quotes
    DQuote  = 12 * State::COUNT as u16,  // "
    SQuote  = 13 * State::COUNT as u16,  // '
    BQuote  = 14 * State::COUNT as u16,  // `
    // Isolated characters
    Tilde   = 15 * State::COUNT as u16,  // ~
    Bang    = 16 * State::COUNT as u16,  // !
    At      = 17 * State::COUNT as u16,  // @
    Hash    = 18 * State::COUNT as u16,  // #
    Dollar  = 19 * State::COUNT as u16,  // $
    Percent = 20 * State::COUNT as u16,  // %
    Caret   = 21 * State::COUNT as u16,  // ^
    Amper   = 22 * State::COUNT as u16,  // &
    Star    = 23 * State::COUNT as u16,  // *
    Minus   = 24 * State::COUNT as u16,  // -
    Equal   = 25 * State::COUNT as u16,  // =
    Plus    = 26 * State::COUNT as u16,  // +
    BSlash  = 27 * State::COUNT as u16,  // \
    Pipe    = 28 * State::COUNT as u16,  // |
    Semi    = 29 * State::COUNT as u16,  // ;
    Colon   = 30 * State::COUNT as u16,  // :
    Comma   = 31 * State::COUNT as u16,  // ,
    Lt      = 32 * State::COUNT as u16,  // <
    Dot     = 33 * State::COUNT as u16,  // .
    Gt      = 34 * State::COUNT as u16,  // >
    Slash   = 35 * State::COUNT as u16,  // /
    Quest   = 36 * State::COUNT as u16,  // ?
    // Unlikely
    Other   = 37 * State::COUNT as u16,  // any code point not in another category
}

impl EqClass {
    /// Count of character equivalence classes.
    const COUNT: usize = Other as usize / State::COUNT + 1;
}

impl ConstDefault for EqClass {
    const DEFAULT: Self = Eof;
}

/// Map from UTF-8 byte to character equivalence class.
static EQ_CLASS_MAP: [EqClass; 256] = [
//
//  7-bit ASCII characters
//  x0      x1      x2      x3      x4      x5      x6      x7      CHARS
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // ........
    Other,  Space,  Lf,     Other,  Other,  Cr,     Other,  Other,  // .tn..r..
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // ........
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // ........
    Space,  Bang,   DQuote, Hash,   Dollar, Percent,Amper,  SQuote, //  !"#$%&'
    LParen, RParen, Star,   Plus,   Comma,  Minus,  Dot,    Slash,  // ()*+,-./
    Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  // 01234567
    Digit,  Digit,  Colon,  Semi,   Lt,     Equal,  Gt,     Quest,  // 89:;<=>?
    At,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // @ABCDEFG
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // HIJKLMNO
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // PQRSTUVW
    Id,     Id,     Id,     LSquare,BSlash, RSquare,Caret,  Id,     // XYZ[\]^_
    BQuote, Id,     Id,     Id,     Id,     Id,     Id,     Id,     // `abcdefg
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // hijklmno
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // pqrstuvw
    Id,     Id,     Id,     LCurly, Pipe,   RCurly, Tilde,  Other,  // xyz{|}~. <- DEL
//
//  UTF-8 multibyte sequences
//  0 (8)   1 (9)   2 (A)   3 (B)   4 (C)   5 (D)   6 (E)   7 (F)   RANGE
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 80-87
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 88-8F
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 90-97
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 98-9F
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // A0-A7
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // A8-AF
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // B0-B7
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // B8-BF
    Other,  Other,  Id,     Id,     Id,     Id,     Id,     Id,     // C0-C7
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // C8-CF
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // D0-D7
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // D8-DF
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // E0-E7
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // E8-EF
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // F0-F7
    Id,     Id,     Id,     Id,     Id,     Id,     Other,  Other,  // F8-FF
];

// ----------------------------------------------------------------------------

/// Lexer states
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    /// Normal state.  Any token is possible.
    Normal,

    /// At the begining of a line.
    AtBol,

    /// After a CR.
    AfterCr,

    /// In a comment.
    InComment,
}

impl State {
    /// Count of lexer states.
    const COUNT: usize = InComment as usize + 1;
}

// ----------------------------------------------------------------------------

// Transitions between lexer states.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum TransitionId {
    /// Terminate with success.
    OnEnd,

    /// Terminate with failure.
    OnError,

    /// Transition to `Normal` state.
    ToNormal,
}

impl TransitionId {
    /// Count of transitions.
    const COUNT: usize = ToNormal as usize + 1;
}

/// Lexer state transition table.
static TRANSITION_MAP: [TransitionId; State::COUNT * EqClass::COUNT] = [
//          Normal    AtBol     AfterCr   InComment
// ----------------------------------------------------------------------------
/* Eof   */ OnEnd,    OnEnd,    OnEnd,    OnEnd,
/* Space */ ToNormal, ToNormal, ToNormal, ToNormal,
/* Cr    */ OnError,  OnError,  OnError,  OnError,
/* Lf    */ OnError,  OnError,  OnError,  OnError,
/* Id    */ OnError,  OnError,  OnError,  OnError,
/* Digit */ OnError,  OnError,  OnError,  OnError,

/*   (   */ OnError,  OnError,  OnError,  OnError,
/*   )   */ OnError,  OnError,  OnError,  OnError,
/*   [   */ OnError,  OnError,  OnError,  OnError,
/*   ]   */ OnError,  OnError,  OnError,  OnError,
/*   {   */ OnError,  OnError,  OnError,  OnError,
/*   }   */ OnError,  OnError,  OnError,  OnError,

/*   "   */ OnError,  OnError,  OnError,  OnError,
/*   '   */ OnError,  OnError,  OnError,  OnError,
/*   `   */ OnError,  OnError,  OnError,  OnError,

/*   ~   */ OnError,  OnError,  OnError,  OnError,
/*   !   */ OnError,  OnError,  OnError,  OnError,
/*   @   */ OnError,  OnError,  OnError,  OnError,
/*   #   */ OnError,  OnError,  OnError,  OnError,
/*   $   */ OnError,  OnError,  OnError,  OnError,
/*   %   */ OnError,  OnError,  OnError,  OnError,
/*   ^   */ OnError,  OnError,  OnError,  OnError,
/*   &   */ OnError,  OnError,  OnError,  OnError,
/*   *   */ OnError,  OnError,  OnError,  OnError,
/*   -   */ OnError,  OnError,  OnError,  OnError,
/*   =   */ OnError,  OnError,  OnError,  OnError,
/*   +   */ OnError,  OnError,  OnError,  OnError,
/*   /   */ OnError,  OnError,  OnError,  OnError,
/*   |   */ OnError,  OnError,  OnError,  OnError,
/*   ;   */ OnError,  OnError,  OnError,  OnError,
/*   :   */ OnError,  OnError,  OnError,  OnError,
/*   ,   */ OnError,  OnError,  OnError,  OnError,
/*   <   */ OnError,  OnError,  OnError,  OnError,
/*   .   */ OnError,  OnError,  OnError,  OnError,
/*   >   */ OnError,  OnError,  OnError,  OnError,
/*   /   */ OnError,  OnError,  OnError,  OnError,
/*   ?   */ OnError,  OnError,  OnError,  OnError,

/* Other */ OnError,  OnError,  OnError,  OnError,
];

// ----------------------------------------------------------------------------

/// Transition behavior definitions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Transition {
    state:  State,
    action: Action,
    flags:  u16,
}

/// Transition behavior definitions.
static TRANSITION_LUT: [Transition; TransitionId::COUNT] = [
/* OnEnd    */ Transition { state: Normal, action: Succeed, flags: 0 },
/* OnError  */ Transition { state: Normal, action: Fail,    flags: 0 },
/* ToNormal */ Transition { state: Normal, action: Nop,     flags: 0 },
];

// ----------------------------------------------------------------------------

/// Lexer actions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Action {
    // Do nothing.
    Nop,

    // Terminate successfully.
    Succeed,

    // Terminate unsuccessfully.
    Fail,
}

// ----------------------------------------------------------------------------

/// A byte reader optimized for use by the lexer.
struct Reader<'a, T> where T: 'a + ConstDefault {
    ptr: *const u8,
    end: *const u8,
    map: &'a [T; 256],
}

impl<'a, T> Reader<'a, T> where T: 'a + ConstDefault {
    // Safety: This is a micro-optimization of std::slice::Iter.  The unsafe
    // blocks are equivalent to those in std::slice::Iter and thus have the
    // effective safety.

    #[inline(always)]
    pub fn new(bytes: &'a [u8], map: &'a [T; 256]) -> Self {
        let ptr = bytes.as_ptr();
        let end = unsafe { ptr.add(bytes.len()) };

        Self { ptr, end, map }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.ptr == self.end
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.end as usize - self.ptr as usize
    }

    #[inline(always)]
    pub fn peek(&self) -> T {
        let p = self.ptr;
        if p == self.end {
            T::DEFAULT
        } else {
            unsafe {
                self.map[*p as usize]
            }
        }
    }

    #[inline(always)]
    pub fn next(&mut self) -> T {
        let p = self.ptr;
        if p == self.end {
            T::DEFAULT
        } else {
            unsafe {
                self.ptr = p.offset(1);
                self.map[*p as usize]
            }
        }
    }

    #[inline(always)]
    pub fn advance(&mut self) -> bool {
        let p = self.ptr;
        if p == self.end {
            false
        } else {
            unsafe {
                self.ptr = p.offset(1);
                true
            }
        }
    }
}

// ----------------------------------------------------------------------------

/// A lexical analyzer.  Reads input and yields a stream of lexical tokens.
pub struct Lexer<'a> {
    input: Reader<'a, EqClass>,
    state: State,
}

impl<'a> Lexer<'a> {
    /// Creates a lexical analyzer that takes as input the contents of the
    /// given slice of bytes.
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input: Reader::new(input, &EQ_CLASS_MAP),
            state: Normal,
        }
    }

    /// Gets the next lexical token.
    pub fn next(&mut self) -> Token {
        // Restore saved state and prepare for loop
        let ref mut input = self.input;
        let     mut state = self.state;
        let     mut action;
        let     mut length = 0;

        // Discover next token
        loop {
            let next = input.next();
            let next = TRANSITION_MAP[state as usize + next as usize];
            let next = TRANSITION_LUT[next  as usize];

            state   = next.state;
            action  = next.action;
            length += next.flags & 1u16;

            if action != Nop { break }
        }

        // Save state for subsequent invocation
        self.state = state;

        // Return token
        match action {
            Nop     => unreachable!(),
            Succeed => Token::Eof,
            Fail    => Token::Error,
        }
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reader_empty() {
        let mut reader = Reader::new(b"", &EQ_CLASS_MAP);

        assert_eq!( reader.is_empty(), true  );
        assert_eq!( reader.peek(),     Eof   );
        assert_eq!( reader.next(),     Eof   );
        assert_eq!( reader.advance(),  false );
    }

    #[test]
    fn reader_next() {
        let mut reader = Reader::new(b"X+1", &EQ_CLASS_MAP);

        assert_eq!( reader.is_empty(), false );
        assert_eq!( reader.peek(),     Id    );
        assert_eq!( reader.next(),     Id    );

        assert_eq!( reader.is_empty(), false );
        assert_eq!( reader.peek(),     Plus  );
        assert_eq!( reader.next(),     Plus  );

        assert_eq!( reader.is_empty(), false );
        assert_eq!( reader.peek(),     Digit );
        assert_eq!( reader.next(),     Digit );

        assert_eq!( reader.is_empty(), true  );
        assert_eq!( reader.peek(),     Eof   );
        assert_eq!( reader.next(),     Eof   );
    }

    #[test]
    fn reader_advance() {
        let mut reader = Reader::new(b"X+1", &EQ_CLASS_MAP);

        assert_eq!( reader.is_empty(), false );
        assert_eq!( reader.peek(),     Id    );
        assert_eq!( reader.advance(),  true  );

        assert_eq!( reader.is_empty(), false );
        assert_eq!( reader.peek(),     Plus  );
        assert_eq!( reader.advance(),  true  );

        assert_eq!( reader.is_empty(), false );
        assert_eq!( reader.peek(),     Digit );
        assert_eq!( reader.advance(),  true  );

        assert_eq!( reader.is_empty(), true  );
        assert_eq!( reader.peek(),     Eof   );
        assert_eq!( reader.advance(),  false );
    }

    #[test]
    fn lexer_empty() {
        let mut lexer = Lexer::new(b"");

        assert_eq!( lexer.next(), Token::Eof );
    }
}

