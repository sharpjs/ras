# The `ras` Language

```text
This file is part of ras, an assembler.
Copyright 2022 Jeffrey Sharp

SPDX-License-Identifier: GPL-3.0-or-later

ras is free software: you can redistribute it and/or modify it
under the terms of the GNU General Public License as published
by the Free Software Foundation, either version 3 of the License,
or (at your option) any later version.

ras is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See
the GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with ras.  If not, see <http://www.gnu.org/licenses/>.
```

## Lexical Structure

### Number Format

```text
[base] significand [exponent]
─┬──── ───┬─────── ──┬───────
 ├─ b'    ├─ 1       ├─ p1
 ├─ o'    ├─ 1.      ├─ p+1
 ├─ d'    ├─ 1.1     └─ p-1
 └─ x'    └─  .1
```

### Escape Sequences

Sequence | UTF-8   | Name  | Description
---------|---------|:------|:-----------
`\0`     | `00`    | `NUL` | null character
`\a`     | `07`    | `BEL` | bell, alert
`\b`     | `08`    | `BS`  | backspace
`\t`     | `09`    | `HT`  | horizontal tab
`\n`     | `0A`    | `LF`  | line feed, newline
`\v`     | `0B`    | `VT`  | vertical tab
`\f`     | `0C`    | `FF`  | form feed
`\r`     | `0D`    | `CR`  | carriage return
`\e`     | `1B`    | `ESC` | escape
`\s`     | `20`    | ` `   | space
`\"`     | `22`    | `"`   | double quote
`\'`     | `27`    | `'`   | single quote
`\\`     | `5C`    | `\`   | backslash
`\d`     | `7F`    | `DEL` | delete

## Grammar

### Operator Precedence and Associativity

| Operators                          |Prec| Assoc | Arity | Signedness        | Description
|:-----------------------------------|---:|:-----:|------:|:-----------------:|:-----------
| `( )` `[ ]` `{ }`                  | 15 | non   |     1 |                   | group
| `@`                                | 15 | right |     2 |                   | alias
| `:`                                | 14 | right |     2 |                   | composition
| `x++` `x--` `f()` `x[]`            | 13 | left  |     1 |                   | unary postfix
| `~` `!` `+x` `-x` `%x` `++x` `--x` | 12 | right |     1 |                   | unary prefix
| `*` `/` `%`                        | 11 | left  |     2 | `*` `/` `%`       | multiplicative
| `+` `-`                            | 10 | left  |     2 |                   | additive
| `<<` `>>`                          |  9 | left  |     2 | `>>`              | shift
| `&`                                |  8 | left  |     2 |                   | bitwise AND
| `^`                                |  7 | left  |     2 |                   | bitwise XOR
| `\|`                               |  6 | left  |     2 |                   | bitwise OR
| `==` `!=` `<` `>` `<=` `>=`        |  5 | left  |     2 | `<` `>` `<=` `>=` | comparison
| `&&`                               |  4 | left  |     2 |                   | logical AND
| `^^`                               |  3 | left  |     2 |                   | logical XOR
| `\|\|`                             |  2 | left  |     2 |                   | logical OR
| `=` `*=` `/=` `%=`<br>`+=` `-=` `<<=` `>>=`<br>`&=` `^=` `\|=` `&&=` `^^=` `\|\|=` | 1 | right | 2 | some<sup>1</sup> | assignment
| `+:` `%:`                          | 0  | right |     1 |                   | signedness

<sup>1</sup> Compound assignment operator signedness behavior matches that of
the corresponding non-assignment operator.

## Directives

```
.nop        Do nothing.
.signed     Set default signedness to signed.
.unsigned   Set default signedness to unsigned.
```
