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
`\s`     | `20`    | ` `   | space
`\"`     | `22`    | `"`   | double quote
`\'`     | `27`    | `'`   | single quote
`\\`     | `5C`    | `\`   | backslash
`\d`     | `7F`    | `DEL` | delete

## Grammar

### Operator Precedence and Associativity

| Operators                          |Prec|Assoc| Arity | Signedness        | Description
|:-----------------------------------|---:|:---:|------:|:-----------------:|:-----------
| `( )` `[ ]` `{ }`                  | 17 |  —  |     1 |                   | group
| `@`                                | 16 |  R⯈ |     2 |                   | alias
| `x++` `x--` `f()` `x[]`            | 15 | ⯇L  |     1 |                   | unary postfix
| `~` `!` `%x` `+x` `-x` `++x` `--x` | 14 |  R⯈ |     1 |                   | unary prefix
| `*` `/` `%`                        | 13 | ⯇L  |     2 | `*` `/` `%`       | multiplicative
| `+` `-`                            | 12 | ⯇L  |     2 |                   | additive
| `<<` `>>`                          | 11 | ⯇L  |     2 | `>>`              | shift
| `&`                                | 10 | ⯇L  |     2 |                   | bitwise AND
| `^`                                |  9 | ⯇L  |     2 |                   | bitwise XOR
| `\|`                               |  8 | ⯇L  |     2 |                   | bitwise OR
| `==` `!=` `<` `>` `<=` `>=`        |  7 | ⯇L  |     2 | `<` `>` `<=` `>=` | comparison
| `&&`                               |  6 | ⯇L  |     2 |                   | logical AND
| `^^`                               |  5 | ⯇L  |     2 |                   | logical XOR
| `\|\|`                             |  4 | ⯇L  |     2 |                   | logical OR
| `=` `*=` `/=` `%=`<br>`+=` `-=` `<<=` `>>=`<br>`&=` `^=` `\|=` `&&=` `^^=` `\|\|=` | 3 | R⯈ | 2 | some<sup>1</sup> | assignment
| `~`                                |  2 |  —  |     2 |                   | range
| `:`                                |  1 |  R⯈ |     2 |                   | composition
| `%:` `+:`                          |  0 |  R⯈ |     1 |                   | signedness
|                                    |    |     |       |                   |
| `$`                                | -1 |  —  |     2 |                   | duplication
| `,`                                | -2 |  R⯈ |     2 |                   | sequencing

<sup>1</sup> Compound assignment operator signedness behavior matches that of
the corresponding non-assignment operator.

### Formal Specification in [ABNF](https://www.rfc-editor.org/rfc/rfc5234.html)

```asm
  .macro a=1, b=2, !c=3
```

```abnf
block           = *stmt

stmt            = EOS ; omitted from AST
                / label
                / define-stmt
                / macro-stmt
                / directive

label           = IDENT label-kind

label-kind      = ":"  ; local or private
                / "::" ; public
                / ":?" ; weak

directive       = IDENT [args] EOS
                ; except ".define" / ".macro"

args            = arg *( "," arg )

arg             = "?"
                / expr
                / arg [ "$" expr ]

expr            = atom
                / "(" expr ")"
                / "[" expr "]" ["!"]
                / "{" block "}"
                / prefix-op expr      ; subject to precedence
                / expr postfix-op     ; subject to precedence
                / expr infix-op expr  ; subject to precedence

atom            = IDENT / INT / FLOAT / STR / CHAR

prefix-op       = "++" / "--" / "~" / "!" / "%" / "+" / "-" / "%:" / "+:"

postfix-op      = "++" / "--"

infix-op        = "@"
                / "*"  / "/"   / "%"
                / "+"  / "-"
                / "<<" / ">>"
                / "&"  / "^"   / "|"
                / "==" / "!="  / "<"   / ">"  / "<=" / ">="
                / "&&" / "^^"  / "||"
                / "="  / "*="  / "/="  / "%="
                       / "+="  / "-="
                       / "<<=" / ">>="
                       / "&="  / "^="  / "|="
                       / "&&=" / "^^=" / "||="
                / "~"
                / ":"

define-stmt     = ".define" IDENT [ "(" [macro-args] ")" ] "=" *token-tree EOS

macro-stmt      = ".macro" IDENT [macro-args] EOS

macro-args      = macro-arg *( "," macro-arg )

macro-arg       = *macro-arg-flag IDENT [ "=" *token-tree-nc ]

macro-arg-flag  = "!" ; eager evaluation
                / "+" ; remaining arguments

token-tree      = token-tree-nc / ","

token-tree-nc   = atom
                / prefix-op
                / postfix-op
                / infix-op
                / label-kind
                / "?" / "$"
                / "(" *token-tree ")"
                / "[" *token-tree "]"
                / "{" *token-tree "}"
```

## Directives

Name        | Description
:-----------|:-----------------------------------------------------------------
`.end`      | Ends the current scope.
`.define`   | Defines a function-like macro.
`.macro`    | Defines a statement-like macro.
`.nop`      | Does nothing.
`.block`    | Renders a block.
`.signed`   | Sets default signedness to signed.
`.unsigned` | Sets default signedness to unsigned.

### General

#### .end

> ```
> .end [ <name> ]
> ```
>
> Ends the current scope.  If an identifier is provided, the assembler
> verifies that it matches the name of the ending scope.

### Signedness

The operators `*` `/` `%` `>>` `<` `>` `<=` `>=` behave differently depending
on the signedness of operands.  The operators `%` `+` `-` `%:` `+:` override
the default signedness.

#### .signed

```
.signed
```

Sets the default signedness to **signed** for subsequent statements.

#### .unsigned

```
.unsigned
```

Sets the default signedness to **unsigned** for subsequent statements.

### Macros

#### .define

```
.define <name> = <value>
```

Defines an inline macro without parameters which expands to the specified
value.

```
.define <name> ( <params> ) = <value>
```

Defines an inline macro with parameters which expands to the specified value.

`<name>` is the macro name and must be an identifier.

`<params>` is a comma-separated list of zero or more parameters.

`<value>` is any set of token trees and may be empty.

#### .macro

```
.macro <name> <params>
    ...
.end
```

Defines a directive-like macro.

#### Macro Parameters

Each macro parameter has the form

```
[ ! | + ] <name> [ = <value> ]
```

where `name` is the parameter name, and `value` is an optional default value.

Optional prefixes, which may appear in any order, alter the behavior of the
parameter:
- `!` causes the parameter to use eager evaluation rather than lazy.
- `+` causes the parameter to capture all remaining arguments and the commas
      separating them.  This prefix may appear only on the last argument.

If a parameter has a default value, the parameter is optional.  Otherwise, the
parameter is required.  The default value may be empty.
