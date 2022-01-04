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

use crate::lang::input::LogicalChar;

///! Main lexer.

use super::*;

// ----------------------------------------------------------------------------

/// Logical characters recognized by the main lexer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
enum Char {
    // space, line endings
    Space   = char( 0), // \s \t \v \f
    Cr      = char( 1), // \r
    Lf      = char( 2), // \n
    // identifiers, numbers
    Ident   = char( 3), // A-Z a-z . and all code points above U+007F
    Digit   = char( 4), // 0-9
    // open/close pairs
    LParen  = char( 5), // (
    RParen  = char( 6), // )
    LSquare = char( 7), // [
    RSquare = char( 8), // ]
    LCurly  = char( 9), // {
    RCurly  = char(10), // }
    // quotes
    DQuote  = char(11), // "
    SQuote  = char(12), // '
    // isolated characters, ordered by descending frequency
    Comma   = char(13), // ,
    Hash    = char(14), // #
    Equal   = char(15), // =
    Plus    = char(16), // +
    Minus   = char(17), // -
    Amp     = char(18), // &
    Pipe    = char(19), // |
    Caret   = char(20), // ^
    Lt      = char(21), // <
    Gt      = char(22), // >
    Tilde   = char(23), // ~
    Bang    = char(24), // !
    Star    = char(25), // *
    Slash   = char(26), // /
    Percent = char(27), // %
    Semi    = char(28), // ;
    Colon   = char(29), // :
    Quest   = char(30), // ?
    Dollar  = char(31), // $
    At      = char(32), // @
    BSlash  = char(33), // \
    // rare
    Eof     = char(34), // end of file
    Other   = char(35), // everything else // <- COUNT references this
}

// Helper to define Char variants
const fn char(n: u16) -> u16 {
    n * State::COUNT as u16
}

impl Char {
    /// Count of logical characters.
    const COUNT: usize = Self::Other as usize / State::COUNT + 1;
}

impl LogicalChar for Char {
    const NON_ASCII: Self = Self::Ident;
    const EOF:       Self = Self::Eof;
}

/// Mapping of 7-bit ASCII to logical characters.
static CHARS: [Char; 128] = {
    use Char::*;
    const __: Char = Other;
[
//  x0       x1       x2       x3       x4       x5       x6       x7
//  x8       x9       xA       xB       xC       xD       xE       xF
    __,      __,      __,      __,      __,      __,      __,      __,      // 0x │········│
    __,      Space,   Lf,      Space,   Space,   Cr,      __,      __,      // 0x │·tnvfr··│
    __,      __,      __,      __,      __,      __,      __,      __,      // 1x │········│
    __,      __,      __,      __,      __,      __,      __,      __,      // 1x │········│
    Space,   Bang,    DQuote,  Hash,    Dollar,  Percent, Amp,     SQuote,  // 2x │ !"#$%&'│
    LParen,  RParen,  Star,    Plus,    Comma,   Minus,   Ident,   Slash,   // 2x │()*+,-./│
    Digit,   Digit,   Digit,   Digit,   Digit,   Digit,   Digit,   Digit,   // 3x │01234567│
    Digit,   Digit,   Colon,   Semi,    Lt,      Equal,   Gt,      Quest,   // 3x │89:;<=>?│
    At,      Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   // 4x │@ABCDEFG│
    Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   // 4x │HIJKLMNO│
    Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   // 5x │PQRSTUVW│
    Ident,   Ident,   Ident,   LSquare, BSlash,  RSquare, Caret,   Ident,   // 5x │XYZ[\]^_│
    __,      Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   // 6x │`abcdefg│
    Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   // 6x │hijklmno│
    Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   // 7x │pqrstuvw│
    Ident,   Ident,   Ident,   LCurly,  Pipe,    RCurly,  Tilde,   __,      // 7x │xyz{|}~░│
]};

// ----------------------------------------------------------------------------

/// Main lexer states.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    /// Normal state.  Any token is possible.
    Normal,

    /// In a comment.
    Comment,

    /// After `=`.
    AfterEqual, // <- COUNT references this
}

impl State {
    /// Count of main lexer states.
    const COUNT: usize = Self::AfterEqual as usize + 1;
}

// ----------------------------------------------------------------------------

// Main lexer transitions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Transition {
    /// Consume the current input byte and continue scanning in `Normal` state.
    Normal,

    /// Handle a CR or CR+LF newline.
    /// - Consume the current input byte.
    /// - Consume the next input byte if it is a line feed.
    /// - Emit an `Eos` token.
    /// - Increment the line number for subsequent tokens.
    CrEol,

    /// Handle a LF newline.
    /// - Consume the current input byte.
    /// - Emit an `Eos` token.
    /// - Increment the line number for subsequent tokens.
    LfEol,

    /// Consume the current input byte and continue scanning in `Comment`
    /// state.
    Comment,

    /// Scan an identifier or a label.
    Ident,

    /// Scan a macro parameter.
    Param,

    /// Scan a decimal numeric literal.
    IntDec,

    /// Scan a string literal.
    Str,

    /// Scan a character literal.
    Char,

    /// Consume the current input byte and emit a `LogNot` token.
    LogNot,

    /// Consume the current input byte and emit a `BitNot` token.
    BitNot,

    /// Consume the current input byte and emit an `Unknown` token.
    Unknown,

    /// Consume the current input byte and emit a `LParen` token.
    LParen,

    /// Consume the current input byte and emit a `RParen` token.
    RParen,

    /// Consume the current input byte and emit a `LSquare` token.
    LSquare,

    /// Consume the current input byte and emit a `RSquare` token.
    RSquare,

    /// Consume the current input byte and emit a `LCurly` token.
    LCurly,

    /// Consume the current input byte and emit a `RCurly` token.
    RCurly,

    /// Consume the current input byte and emit a `Comma` token.
    Comma,

    /// Consume the current input byte and emit a `Colon` token.
    Colon,

    /// Consume the current input byte and continue scanning in `AfterEqual`
    /// state.
    Equal_,

    /// Emit an `Assign` token.
    Assign,

    /// Consume the current input byte and emit an `Eq` token.
    Eq,

    /// Emit an `Eof` token.
    End,

    /// Handle a lexical error.
    /// - Record a lexical error at the current input position.
    /// - Consume the current input byte.
    /// - Continue scanning in `Normal` state.
    Error,
}

impl Transition {
    /// Returns the action and token length increment for the transition.
    fn decode(self) -> (Action, usize) {
        use Action::*;
        use State      as S;
        use Token      as T;
        use Transition as X;

        match self {
            //                     Action    Arguments    +Length
            // Whitespace        ----------------------------------
            X::Normal         => ( Continue  (S::Normal),       0 ),
            X::CrEol          => ( ScanCrLf  ,                  1 ),
            X::LfEol          => ( ScanLf    ,                  1 ),
            X::Comment        => ( Continue  (S::Comment),      0 ),
            // Numbers           ----------------------------------
            X::Ident          => ( ScanIdent ,                  1 ),
            X::Param          => ( ScanParam ,                  1 ),
            X::IntDec         => ( ScanDec   ,                  1 ),
            X::Str            => ( ScanStr   ,                  1 ),
            X::Char           => ( ScanChar  ,                  1 ),
            // Simple Tokens     ----------------------------------
            X::LogNot         => ( Produce   (T::LogNot),       1 ),
            X::BitNot         => ( Produce   (T::BitNot),       1 ),
            X::Unknown        => ( Produce   (T::Unknown),      1 ),
            X::LParen         => ( Produce   (T::LParen),       1 ),
            X::RParen         => ( Produce   (T::RParen),       1 ),
            X::LSquare        => ( Produce   (T::LSquare),      1 ),
            X::RSquare        => ( Produce   (T::RSquare),      1 ),
            X::LCurly         => ( Produce   (T::LCurly),       1 ),
            X::RCurly         => ( Produce   (T::RCurly),       1 ),
            X::Comma          => ( Produce   (T::Comma),        1 ),
            X::Colon          => ( Produce   (T::Colon),        1 ),
            // Complex Tokens    ----------------------------------
            X::Equal_         => ( Continue  (S::AfterEqual),   1 ),
            X::Assign         => ( Yield     (T::Assign),       0 ),
            X::Eq             => ( Produce   (T::Eq),           1 ),
            // Other             ----------------------------------
            X::End            => ( Yield     (T::Eof),          0 ), // will position be correct?
            X::Error          => ( Error     ,                  0 ),
        }
    }
}

// ----------------------------------------------------------------------------

/// Main lexer actions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Action {
    /// Consume the current input byte and continue scanning in a new state.
    Continue(State),

    // === Tokens ===

    /// Consume the current input byte and emit a token.
    Produce(Token),

    /// Emit a token.
    Yield(Token),

    // === Sublexers ===

    /// Scan a CR or CR+LF end-of-statement.
    ScanCrLf,

    /// Scan a LF end-of-statement.
    ScanLf,

    /// Scan a binary numeric literal.
    ScanBin,

    /// Scan an octal numeric literal.
    ScanOct,

    /// Scan a decimal numeric literal.
    ScanDec,

    /// Scan a hexadecimal numeric literal.
    ScanHex,

    /// Scan a string literal.
    ScanStr,

    /// Scan a character literal.
    ScanChar,

    /// Scan an identifier.
    ScanIdent,

    /// Scan a macro parameter.
    ScanParam,

    // === Miscellaneous ===

    /// Record a lexical error, consume the current input byte, and continue
    /// scanning in `Normal` state.
    Error,
}

// ----------------------------------------------------------------------------

/// Main lexer state transition map.
static TRANSITION_MAP: [Transition; State::COUNT * Char::COUNT] = {
    use Transition::*;
    const __: Transition = Error;
[
//          Normal      Comment     Equal
//          -------------------------------------
/* Space */ Normal,     Comment,    Assign,
/*   Cr  */ CrEol,      CrEol,      Assign,
/*   Lf  */ LfEol,      LfEol,      Assign,

/* Ident */ Ident,      Comment,    Assign,
/* Digit */ IntDec,     Comment,    Assign,

/*   (   */ LParen,     Comment,    Assign,
/*   )   */ RParen,     Comment,    Assign,
/*   [   */ LSquare,    Comment,    Assign,
/*   ]   */ RSquare,    Comment,    Assign,
/*   {   */ LCurly,     Comment,    Assign,
/*   }   */ RCurly,     Comment,    Assign,
/*   "   */ Str,        Comment,    Assign,
/*   '   */ Char,       Comment,    Assign,

/*   ,   */ Comma,      Comment,    Assign,
/*   #   */ Comment,    Comment,    Assign,
/*   =   */ Equal_,     Comment,    Eq,
/*   +   */ __,         Comment,    Assign,
/*   -   */ __,         Comment,    Assign,
/*   &   */ __,         Comment,    Assign,
/*   |   */ __,         Comment,    Assign,
/*   ^   */ __,         Comment,    Assign,
/*   <   */ __,         Comment,    Assign,
/*   >   */ __,         Comment,    Assign,
/*   ~   */ BitNot,     Comment,    Assign,
/*   !   */ LogNot,     Comment,    Assign,
/*   *   */ __,         Comment,    Assign,
/*   /   */ __,         Comment,    Assign,
/*   %   */ __,         Comment,    Assign,
/*   ;   */ __,         Comment,    Assign,
/*   :   */ Colon,      Comment,    Assign,
/*   ?   */ Unknown,    Comment,    Assign,
/*   $   */ Param,      Comment,    Assign,
/*   @   */ __,         Comment,    Assign,
/*   \   */ __,         Comment,    Assign,

/*  Eof  */ End,        End,        Assign,
/* Other */ Error,      Comment,    Assign,
]};

// ----------------------------------------------------------------------------

impl<I: Iterator<Item = u8>> Lexer<I> {
    /// Scans a token.
    pub(super) fn scan_main(&mut self) -> Token {
        use Action::*;

        // Apply deferred info from previous call
        self.line = self.line_next;

        // Every call begins in `Normal` state with no token found
        let mut state   = State::Normal;
        let mut started = 0;
        let mut offset  = 0;
        let mut len     = 0;

        // Scan until a token is found
        let token = loop {
            // Translate input into action
            let (input, _)    = self.input.classify(&CHARS);
            let transition    = TRANSITION_MAP[state as usize + input as usize];
            let (action, inc) = transition.decode(); // inc = 0 or 1

            // Accumulate token length
            len += inc;

            // Detect token start
            //           <- Before Token | At Token Start | After Token Start ->
            // inc:               0      |        1       |       0 or 1
            // starting:       all 0s    |     all 1s     |       all 0s
            // started:        all 0s    |     all 1s     |       all 1s
            let starting = (inc & !started).wrapping_neg();
            started |= starting;
            offset  |= starting & self.input.position();

            // Perform action
            match action {
                Continue (s) => state = self.transition(s),
                Error        => state = self.add_error(),
                Yield    (t) => break t,
                Produce  (t) => break self.produce(t),
                ScanCrLf     => break self.scan_crlf(),
                ScanLf       => break self.scan_lf(),
                ScanBin      => break self.scan_bin(),
                ScanOct      => break self.scan_oct(),
                ScanDec      => break self.scan_dec(),
                ScanHex      => break self.scan_hex(),
                ScanStr      => break self.scan_str(),
                ScanChar     => break self.scan_char(),
                ScanIdent    => break self.scan_ident(),
                ScanParam    => break self.scan_param(),
            }
        };

        // Yield token
        self.offset = offset;
        self.len    = len;
        token
    }

    #[inline]
    fn transition(&mut self, s: State) -> State {
        self.input.advance();
        s
    }

    #[inline]
    fn produce(&mut self, tok: Token) -> Token {
        self.input.advance();
        tok
    }

    #[inline]
    fn scan_crlf(&mut self) -> Token {
        self.input.advance();
        self.input.advance_if(b'\n');
        self.line_next += 1;
        Token::Eos
    }

    #[inline]
    fn scan_lf(&mut self) -> Token {
        self.input.advance();
        self.line_next += 1;
        Token::Eos
    }

    #[inline]
    fn scan_bin(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Int
    }

    #[inline]
    fn scan_oct(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Int
    }

    #[inline]
    fn scan_dec(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Int
    }

    #[inline]
    fn scan_hex(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Int
    }

    #[inline]
    fn scan_str(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Str
    }

    #[inline]
    fn scan_char(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Char
    }

    #[inline]
    fn scan_ident(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Ident
    }

    #[inline]
    fn scan_param(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Param
    }

    fn add_error(&mut self) -> State {
        // add error here
        self.input.advance();
        State::Normal
    }
}
