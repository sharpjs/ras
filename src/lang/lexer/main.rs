// This file is part of ras, an assembler.
// Copyright 2022 Jeffrey Sharp
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
    // space, newlines
    Space   = char( 0), // \s\t
    Cr      = char( 1), // \r
    Lf      = char( 2), // \n
    // identifiers, numbers
    Ident   = char( 3), // A-Za-z., code points above U+007F
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
    Pct     = char(27), // %
    Semi    = char(28), // ;
    Colon   = char(29), // :
    Quest   = char(30), // ?
    Dollar  = char(31), // $
    At      = char(32), // @
    BSlash  = char(33), // \
    // rare
    Eof     = char(34), // end of file
    Other   = char(35), // everything else
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
//  x0      x1      x2      x3      x4      x5      x6      x7
//  x8      x9      xA      xB      xC      xD      xE      xF
    __,     __,     __,     __,     __,     __,     __,     __,     // 0x │········│
    __,     Space,  Lf,     __,     __,     Cr,     __,     __,     // 0x │·tn··r··│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    Space,  Bang,   DQuote, Hash,   Dollar, Pct,    Amp,    SQuote, // 2x │ !"#$%&'│
    LParen, RParen, Star,   Plus,   Comma,  Minus,  Ident,  Slash,  // 2x │()*+,-./│
    Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  // 3x │01234567│
    Digit,  Digit,  Colon,  Semi,   Lt,     Equal,  Gt,     Quest,  // 3x │89:;<=>?│
    At,     Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 4x │@ABCDEFG│
    Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 4x │HIJKLMNO│
    Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 5x │PQRSTUVW│
    Ident,  Ident,  Ident,  LSquare,BSlash, RSquare,Caret,  Ident,  // 5x │XYZ[\]^_│
    __,     Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 6x │`abcdefg│
    Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 6x │hijklmno│
    Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 7x │pqrstuvw│
    Ident,  Ident,  Ident,  LCurly, Pipe,   RCurly, Tilde,  __,     // 7x │xyz{|}~░│
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
}

impl State {
    /// Count of main lexer states.
    const COUNT: usize = Self::Comment as usize + 1;
}

// ----------------------------------------------------------------------------

// Transition IDs.  Each ID is an index into `TRANSITION_LUT`, which contains
// the details of the transition.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum TransitionId {
    /// Transition to `Normal` state, consume the current input byte, and
    /// continue scanning.
    Normal,

    /// Handle a CR or CR+LF newline.
    /// - Transition to `Normal` state.
    /// - Consume the current input byte.
    /// - Consume the subsequent input byte if it is a line feed.
    /// - Emit an `Eos` token.
    /// - Increment the line number for subsequent tokens.
    CrEol,

    /// Handle a LF newline.
    /// - Transition to `Normal` state.
    /// - Consume the current input byte.
    /// - Emit an `Eos` token.
    /// - Increment the line number for subsequent tokens.
    LfEol,

    /// Transition to `Comment` state, consume the current input byte, and
    /// continue scanning.
    Comment,

    /// Transition to `Normal` state and enter the decimal numeric literal sublexer.
    IntDec,

    /// Transition to `Normal` state, consume the current input byte, and emit a `LParen` token.
    LParen,

    /// Transition to `Normal` state, consume the current input byte, and emit a `RParen` token.
    RParen,

    /// Transition to `Normal` state, consume the current input byte, and emit a `LSquare` token.
    LSquare,

    /// Transition to `Normal` state, consume the current input byte, and emit a `RSquare` token.
    RSquare,

    /// Transition to `Normal` state, consume the current input byte, and emit a `LCurly` token.
    LCurly,

    /// Transition to `Normal` state, consume the current input byte, and emit a `RCurly` token.
    RCurly,

    /// Transition to `Normal` state, consume the current input byte, and emit a `Comma` token.
    Comma,

    /// Transition to `Normal` state, consume the current input byte, and emit a `Colon` token.
    Colon,

    /// Record a lexical error at the current input position.  Consume the
    /// current input byte and continue scanning.
    Error,

    /// Transition to `Normal` state and emit an `Eof` token.
    End,
}

impl TransitionId {
    /// Count of transition IDs.
    const COUNT: usize = Self::End as usize + 1;
}

// ----------------------------------------------------------------------------

/// Main lexer transition details.
#[derive(Clone, Copy, Debug)]
struct Transition {
    state:  State,  // 1 byte
    action: Action, // 2 bytes
    flags:  u8,     // 1 byte
    // 0b000000LT
    //         │└Token increment
    //         └─Line increment
}

impl Transition {
    #[inline]
    fn token_inc(&self) -> usize {
        (self.flags & 1) as usize
    }

    #[inline]
    fn line_inc(&self) -> u32 {
        (self.flags >> 1) as u32
    }
}

/// Main lexer transition details in order by transition ID.
static TRANSITION_LUT: [Transition; TransitionId::COUNT] = {
    use Action::*;
    use State::*;
    use Token as T;
    use TransitionId as X;
    const fn t(_: TransitionId, state: State, action: Action, flags: u8) -> Transition {
        Transition { state, action, flags }
    }
[
//                                                                   +len┐
//    TransitionId      NewState    Action      Args              +line┐ │
// ----------------------------------------------------------------------------
// Whitespace                                                          │ │
    t(X::Normal,        Normal,     Consume,                        0b_0_0),
    t(X::CrEol,         Normal,     ScanCrLfEos,                    0b_1_1),
    t(X::LfEol,         Normal,     Produce     (T::Eos),           0b_1_1),
    t(X::Comment,       Comment,    Consume,                        0b_0_0),
// Numbers
    t(X::IntDec,        Normal,     ScanDec,                        0b_0_0),
// Simple Tokens
    t(X::LParen,        Normal,     Produce     (T::LParen),        0b_0_1),
    t(X::RParen,        Normal,     Produce     (T::RParen),        0b_0_1),
    t(X::LSquare,       Normal,     Produce     (T::LSquare),       0b_0_1),
    t(X::RSquare,       Normal,     Produce     (T::RSquare),       0b_0_1),
    t(X::LCurly,        Normal,     Produce     (T::LCurly),        0b_0_1),
    t(X::RCurly,        Normal,     Produce     (T::RCurly),        0b_0_1),
    t(X::Comma,         Normal,     Produce     (T::Comma),         0b_0_1),
    t(X::Colon,         Normal,     Produce     (T::Colon),         0b_0_1),
    t(X::Error,         Normal,     Error,                          0b_0_0),
    t(X::End,           Normal,     Yield       (T::Eof),           0b_0_0),
]};

// ----------------------------------------------------------------------------

/// Main lexer actions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Action {
    /// Consume the current input byte and continue scanning.
    Consume,

    // === Tokens ===

    /// Consume the current input byte and yield a token.
    Produce(Token),

    /// Yield a token.
    Yield(Token),

    // === Sublexers ===

    /// Scan a CR or CR-LF end-of-statement.
    ScanCrLfEos,

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

    /// Record a lexical error.
    Error,
}

// ----------------------------------------------------------------------------

/// Main lexer state transition map.
static TRANSITION_MAP: [TransitionId; State::COUNT * Char::COUNT] = {
    use TransitionId::*;
[
//          Normal      Comment
//          -------------------------------------
/* Space */ Normal,     Comment,
/*   Cr  */ CrEol,      CrEol,
/*   Lf  */ LfEol,      LfEol,

/* Ident */ Error,      Comment,
/* Digit */ IntDec,     Comment,

/*   (   */ LParen,     Comment,
/*   )   */ RParen,     Comment,
/*   [   */ LSquare,    Comment,
/*   ]   */ RSquare,    Comment,
/*   {   */ LCurly,     Comment,
/*   }   */ RCurly,     Comment,
/*   "   */ Error,      Comment,
/*   '   */ Error,      Comment,

/*   ,   */ Comma,      Comment,
/*   #   */ Comment,    Comment,
/*   =   */ Error,      Comment,
/*   +   */ Error,      Comment,
/*   -   */ Error,      Comment,
/*   &   */ Error,      Comment,
/*   |   */ Error,      Comment,
/*   ^   */ Error,      Comment,
/*   <   */ Error,      Comment,
/*   >   */ Error,      Comment,
/*   ~   */ Error,      Comment,
/*   !   */ Error,      Comment,
/*   *   */ Error,      Comment,
/*   /   */ Error,      Comment,
/*   %   */ Error,      Comment,
/*   ;   */ Error,      Comment,
/*   :   */ Colon,      Comment,
/*   ?   */ Error,      Comment,
/*   $   */ Error,      Comment,
/*   @   */ Error,      Comment,
/*   \   */ Error,      Comment,

/*  Eof  */ End,        End,
/* Other */ Error,      Comment,
]};

// ----------------------------------------------------------------------------

impl<I: Iterator<Item = u8>> Lexer<I> {
    /// Scans a token.
    pub(super) fn scan_main(&mut self) -> Token {
        use Action::*;

        let mut state = State::Normal;

        loop {
            let next = self.input.classify(&CHARS).0;
            let next = TRANSITION_MAP[state as usize + next as usize];
            let next = TRANSITION_LUT[next  as usize];

            state = next.state;

            match next.action {
                Consume      =>       self.consume(),
                Produce(tok) => break self.produce(tok),
                Yield  (tok) => break tok,
                ScanCrLfEos  => break self.scan_cr(),
                ScanBin      => break self.scan_bin(),
                ScanOct      => break self.scan_oct(),
                ScanDec      => break self.scan_dec(),
                ScanHex      => break self.scan_hex(),
                ScanStr      => break self.scan_str(),
                ScanChar     => break self.scan_char(),
                ScanIdent    => break self.scan_ident(),
                ScanParam    => break self.scan_param(),
                Error        =>       self.add_error(),
            }

            self.input.advance();
        }
    }

    fn consume(&mut self) {
        self.input.advance()
    }

    fn produce(&mut self, tok: Token) -> Token {
        self.input.advance();
        tok
    }

    fn scan_cr(&mut self) -> Token {
        self.input.advance();
        self.input.advance_if(b'\n');
        Token::Eos
    }

    fn scan_bin(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Int
    }

    fn scan_oct(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Int
    }

    fn scan_dec(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Int
    }

    fn scan_hex(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Int
    }

    fn scan_str(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Str
    }

    fn scan_char(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Char
    }

    fn scan_ident(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Ident
    }

    fn scan_param(&mut self) -> Token {
        self.input.advance(); // TODO: invoke sublexer here
        Token::Param
    }

    fn add_error(&self) { }
}
