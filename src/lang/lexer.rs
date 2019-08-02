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

use std::fmt::{Debug, Formatter, Result};
use std::marker::PhantomData;
use std::slice;

use crate::lang::token::Token;
use crate::util::ConstDefault;

use self::Action::*;
use self::EqClass::*;
use self::State::*;
use self::TransitionId::*;

// ---------------------------------------------------------------------------- 

macro_rules! eq_class {
    ($i:expr) => ($i * State::COUNT as u16)
}

/// Character equivalence classes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
enum EqClass {
    // Variants are in order roughly by descending frequency, except that
    // groups of related variants are kept contiguous.

    // space, newlines
    Space   = eq_class!( 0), // \s\t
    Cr      = eq_class!( 1), // \r
    Lf      = eq_class!( 2), // \n
    // identifiers, numbers
    Id      = eq_class!( 3), // A-Za-z., code points above U+007F
    LetB    = eq_class!( 4), // Bb
    LetD    = eq_class!( 5), // Dd
    LetO    = eq_class!( 6), // Oo
    LetX    = eq_class!( 7), // Xx
    LetHex  = eq_class!( 8), // AaCcEeFf
    Digit   = eq_class!( 9), // 0-9
    Under   = eq_class!(10), // _
    // open/close pairs
    LParen  = eq_class!(11), // (
    RParen  = eq_class!(12), // )
    LSquare = eq_class!(13), // [
    RSquare = eq_class!(14), // ]
    LCurly  = eq_class!(15), // {
    RCurly  = eq_class!(16), // }
    // quotes
    DQuote  = eq_class!(17), // "
    SQuote  = eq_class!(18), // '
    // isolated characters
    Comma   = eq_class!(19), // ,
    Hash    = eq_class!(20), // #
    Equal   = eq_class!(21), // =
    Plus    = eq_class!(22), // +
    Minus   = eq_class!(23), // -
    Amper   = eq_class!(24), // &
    Pipe    = eq_class!(25), // |
    Caret   = eq_class!(26), // ^
    Lt      = eq_class!(27), // <
    Gt      = eq_class!(28), // >
    Tilde   = eq_class!(29), // ~
    Bang    = eq_class!(30), // !
    Star    = eq_class!(31), // *
    Slash   = eq_class!(32), // /
    Percent = eq_class!(33), // %
    Semi    = eq_class!(34), // ;
    Colon   = eq_class!(35), // :
    Quest   = eq_class!(36), // ?
    Dollar  = eq_class!(37), // $
    At      = eq_class!(38), // @    unsure if this will be used
    BSlash  = eq_class!(39), // \
    // rare
    Eof     = eq_class!(40), // end of file
    Other   = eq_class!(41), // code point not in another class
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
    LParen, RParen, Star,   Plus,   Comma,  Minus,  Id,     Slash,  // ()*+,-./
    Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  // 01234567
    Digit,  Digit,  Colon,  Semi,   Lt,     Equal,  Gt,     Quest,  // 89:;<=>?
    At,     LetHex, LetB,   LetHex, LetD,   LetHex, LetHex, Id,     // @ABCDEFG
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     LetO,   // HIJKLMNO
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // PQRSTUVW
    LetX,   Id,     Id,     LSquare,BSlash, RSquare,Caret,  Under,  // XYZ[\]^_
    Other,  LetHex, LetB,   LetHex, LetD,   LetHex, LetHex, Id,     // `abcdefg
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     LetO,   // hijklmno
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // pqrstuvw
    LetX,   Id,     Id,     LCurly, Pipe,   RCurly, Tilde,  Other,  // xyz{|}~. <- DEL
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

/// Lexer states.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    /// Normal state.  Any token is possible.
    Normal,

    /// At the begining of a line.  Any token is possible.
    Bol,

    /// After a CR.
    AfterCr,

    /// In a comment.
    Comment,
}

impl State {
    /// Count of lexer states.
    const COUNT: usize = Comment as usize + 1;
}

// ----------------------------------------------------------------------------

/// Lexer actions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Action {
    // Continue scanning.
    Continue,

    // Terminate successfully.
    Succeed,

    // Terminate unsuccessfully.
    Fail,
}

// ----------------------------------------------------------------------------

// Transition IDs.  Each ID is an index into `TRANSITION_LUT`, which contains
// the details of the transition.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum TransitionId {
    /// Transition to `Normal` state and continue scanning.
    NormCon,

    /// Transition to `Comment` state and continue scanning.
    ComCon,

    /// Terminate with failure.
    Error,

    /// Terminate with success.
    End,
}

impl TransitionId {
    /// Count of transition IDs.
    const COUNT: usize = End as usize + 1;
}

/// Lexer state transition map.
static TRANSITION_MAP: [TransitionId; State::COUNT * EqClass::COUNT] = [
//          Normal    Bol       AfterCr   Comment
// ----------------------------------------------------------------------------
/* \s\t  */ NormCon,  NormCon,  NormCon,  ComCon,
/* \r    */ Error,    Error,    Error,    Error,
/* \n    */ Error,    Error,    Error,    Error,

/*  a-z. */ Error,    Error,    Error,    ComCon,
/*   b   */ Error,    Error,    Error,    ComCon,
/*   d   */ Error,    Error,    Error,    ComCon,
/*   o   */ Error,    Error,    Error,    ComCon,
/*   x   */ Error,    Error,    Error,    ComCon,
/*  acef */ Error,    Error,    Error,    ComCon,
/*  0-9  */ Error,    Error,    Error,    ComCon,
/*   _   */ Error,    Error,    Error,    ComCon,

/*   (   */ Error,    Error,    Error,    ComCon,
/*   )   */ Error,    Error,    Error,    ComCon,
/*   [   */ Error,    Error,    Error,    ComCon,
/*   ]   */ Error,    Error,    Error,    ComCon,
/*   {   */ Error,    Error,    Error,    ComCon,
/*   }   */ Error,    Error,    Error,    ComCon,
/*   "   */ Error,    Error,    Error,    ComCon,
/*   '   */ Error,    Error,    Error,    ComCon,

/*   ,   */ Error,    Error,    Error,    ComCon,
/*   #   */ ComCon,   ComCon,   ComCon,   ComCon,
/*   =   */ Error,    Error,    Error,    ComCon,
/*   +   */ Error,    Error,    Error,    ComCon,
/*   -   */ Error,    Error,    Error,    ComCon,
/*   &   */ Error,    Error,    Error,    ComCon,
/*   |   */ Error,    Error,    Error,    ComCon,
/*   ^   */ Error,    Error,    Error,    ComCon,
/*   <   */ Error,    Error,    Error,    ComCon,
/*   >   */ Error,    Error,    Error,    ComCon,
/*   ~   */ Error,    Error,    Error,    ComCon,
/*   !   */ Error,    Error,    Error,    ComCon,
/*   *   */ Error,    Error,    Error,    ComCon,
/*   /   */ Error,    Error,    Error,    ComCon,
/*   %   */ Error,    Error,    Error,    ComCon,
/*   ;   */ Error,    Error,    Error,    ComCon,
/*   :   */ Error,    Error,    Error,    ComCon,
/*   ?   */ Error,    Error,    Error,    ComCon,
/*   $   */ Error,    Error,    Error,    ComCon,
/*   @   */ Error,    Error,    Error,    ComCon,
/*   \   */ Error,    Error,    Error,    ComCon,

/* Eof   */ End,      End,      End,      End,
/* Other */ Error,    Error,    Error,    ComCon,
];

// ----------------------------------------------------------------------------

/// Lexer transition.
#[derive(Clone, Copy, Debug)]
struct Transition {
    state:  State,
    action: Action,
    flags:  u16,
}

/// Lexer transitions in order by transition ID.
static TRANSITION_LUT: [Transition; TransitionId::COUNT] = [
/* NormCon */ Transition { state: Normal,  action: Continue, flags: 0 },
/* ComCon  */ Transition { state: Comment, action: Continue, flags: 0 },
/* Error   */ Transition { state: Normal,  action: Fail,     flags: 1 },
/* End     */ Transition { state: Normal,  action: Succeed,  flags: 0 },
];

// ----------------------------------------------------------------------------

/// A byte reader optimized for use by the lexer.
#[derive(Clone, Copy)]
struct Reader<'a> {
    ptr: *const u8,
    end: *const u8,
    _lt: PhantomData<&'a ()>,
}

impl<'a> Reader<'a> {
    // Safety: This is a micro-optimization of std::slice::Iter.  The unsafe
    // blocks are equivalent to those in std::slice::Iter and thus have the
    // effective safety.

    #[inline(always)]
    pub fn new(bytes: &'a [u8]) -> Self {
        let ptr = bytes.as_ptr();
        let end = unsafe { ptr.add(bytes.len()) };

        Self { ptr, end, _lt: PhantomData }
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
    pub fn peek<T>(&self, map: &[T; 256]) -> T
        where T: ConstDefault
    {
        let p = self.ptr;
        if p == self.end {
            T::DEFAULT
        } else {
            unsafe {
                map[*p as usize]
            }
        }
    }

    #[inline(always)]
    pub fn next<T>(&mut self, map: &[T; 256]) -> T
        where T: ConstDefault {
        let p = self.ptr;
        if p == self.end {
            T::DEFAULT
        } else {
            unsafe {
                self.ptr = p.offset(1);
                map[*p as usize]
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

    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.ptr, self.len())
        }
    }
}

impl<'a> Debug for Reader<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Reader {:X?}", self.as_slice())
    }
}

// ----------------------------------------------------------------------------

/// A lexical analyzer.  Reads input and yields a stream of lexical tokens.
#[derive(Debug)]
pub struct Lexer<'a> {
    input: Reader<'a>,
    state: State,
}

impl<'a> Lexer<'a> {
    /// Creates a lexical analyzer that takes as input the contents of the
    /// given slice of bytes.
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input: Reader::new(input),
            state: Bol,
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
            let next = input.next(&EQ_CLASS_MAP);
            let next = TRANSITION_MAP[state as usize + next as usize];
            let next = TRANSITION_LUT[next  as usize];

            state   = next.state;
            action  = next.action;
            length += next.flags & 1u16;

            if action != Continue { break }
        }

        // Save state for subsequent invocation
        self.state = state;

        // Return token
        match action {
            Continue => unreachable!(),
            Succeed  => Token::Eof,
            Fail     => Token::Error,
        }
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reader_empty() {
        let mut reader = Reader::new(b"");

        assert_eq!( reader.is_empty(),          true  ); 
        assert_eq!( reader.peek(&EQ_CLASS_MAP), Eof   );
        assert_eq!( reader.next(&EQ_CLASS_MAP), Eof   );
        assert_eq!( reader.advance(),           false );
    }

    #[test]
    fn reader_next() {
        let mut reader = Reader::new(b"X+1");

        assert_eq!( reader.is_empty(),          false );
        assert_eq!( reader.peek(&EQ_CLASS_MAP), LetX  );
        assert_eq!( reader.next(&EQ_CLASS_MAP), LetX  );

        assert_eq!( reader.is_empty(),          false );
        assert_eq!( reader.peek(&EQ_CLASS_MAP), Plus  );
        assert_eq!( reader.next(&EQ_CLASS_MAP), Plus  );

        assert_eq!( reader.is_empty(),          false );
        assert_eq!( reader.peek(&EQ_CLASS_MAP), Digit );
        assert_eq!( reader.next(&EQ_CLASS_MAP), Digit );

        assert_eq!( reader.is_empty(),          true  );
        assert_eq!( reader.peek(&EQ_CLASS_MAP), Eof   );
        assert_eq!( reader.next(&EQ_CLASS_MAP), Eof   );
    }

    #[test]
    fn reader_advance() {
        let mut reader = Reader::new(b"X+1");

        assert_eq!( reader.is_empty(),          false );
        assert_eq!( reader.peek(&EQ_CLASS_MAP), LetX  );
        assert_eq!( reader.advance(),           true  );

        assert_eq!( reader.is_empty(),          false );
        assert_eq!( reader.peek(&EQ_CLASS_MAP), Plus  );
        assert_eq!( reader.advance(),           true  );

        assert_eq!( reader.is_empty(),          false );
        assert_eq!( reader.peek(&EQ_CLASS_MAP), Digit );
        assert_eq!( reader.advance(),           true  );

        assert_eq!( reader.is_empty(),          true  );
        assert_eq!( reader.peek(&EQ_CLASS_MAP), Eof   );
        assert_eq!( reader.advance(),           false );
    }

    #[test]
    fn reader_debug_empty() {
        let reader = Reader::new(b"");

        assert_eq!( format!("{:?}", reader), "Reader []" );
    }

    #[test]
    fn reader_debug_not_empty() {
        let reader = Reader::new(b"X");

        assert_eq!( format!("{:?}", reader), "Reader [58]" );
    }

    #[test]
    fn lexer_empty() {
        let mut lexer = Lexer::new(b"");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_unrecognized() {
        let mut lexer = Lexer::new(b"`");

        assert_eq!( lexer.next(), Token::Error );
    }

    #[test]
    fn lexer_space() {
        let mut lexer = Lexer::new(b" \t \t");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_comment() {
        let mut lexer = Lexer::new(b"# this is a comment");

        assert_eq!( lexer.next(), Token::Eof );
    }
}

