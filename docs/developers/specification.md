# Specification

> **Note**: This specification is a work in progress.

## Introduction

This is a reference manual for the Antimony programming language.

Antimony is a general-purpose language designed with simplicity in mind. It is
strongly typed and supports multiple compile-targets. Programs are constructed
from modules, whose properties allow efficient management of dependencies.

## Notation

The syntax is specified using altered Extended Backus-Naur Form (EBNF):

```
Production  = production_name "=" [ Expression ] "." .
Expression  = Alternative { "|" Alternative } .
Alternative = Term { Term } .
Term        = production_name | token [ "..." token ] | Group | Option | Repetition .
Group       = "(" Expression ")" .
Option      = "[" Expression "]" .
Repetition  = "{" Expression "}" .
```

Productions are expressions constructed from terms and the following operators,
in increasing precedence:

```
|   alternation
()  grouping
[]  option (0 or 1 times)
{}  repetition (0 to n times)
```

Lower-case production names are used to identify lexical tokens. Non-terminals
are in CamelCase. Lexical tokens are enclosed in double quotes `""` or single
quotes `''`.

The form `a ... b` represents the set of characters from `a` through `b` as
alternatives. The horizontal ellipsis `...` is also used elsewhere in the spec
to informally denote various enumerations or code snippets that are not further
specified. The character `…` (as opposed to the three characters `...`) is not a
token of the Antimony language.

## Source Code Representation

Source code is Unicode text encoded in
[UTF-8](https://en.wikipedia.org/wiki/UTF-8). The text is not canonicalized, so
a single accented code point is distinct from the same character constructed
from combining an accent and a letter; those are treated as two code points. For
simplicity, this document will use the unqualified term _character_ to refer to
a Unicode code point in the source text.

Each code point is distinct; for instance, upper and lower case letters are
different characters.

Implementation restriction: For compatibility with other tools, a compiler may
disallow the NUL character (U+0000) in the source text.

### Characters

The following terms are used to denote specific Unicode character classes:

```
newline        = /* the Unicode code point U+000A */ .
unicode_char   = /* an arbitrary Unicode code point except newline */ .
unicode_letter = /* a Unicode code point classified as "Letter" */ .
unicode_digit  = /* a Unicode code point classified as "Number, decimal digit" */ .
```

### Letters and digits

The underscore character `_` (U+005F) is considered a letter.

```
letter        = unicode_letter | "_" .
decimal_digit = "0" ... "9" .
binary_digit  = "0" | "1" .
octal_digit   = "0" ... "7" .
hex_digit     = "0" ... "9" | "A" ... "F" | "a" ... "f" .
```

## Lexical elements

### Comments

Comments serve as program documentation. A comment starts with the character
sequence // and stop at the end of the line.

A comment cannot start inside a string literal, or inside a comment.

### Tokens

Tokens form the vocabulary of the Antimony programming language. There are four
classes: _identifiers_, _keywords_, _operators and punctuation_, and _literals_.
_White space_, formed from spaces (U+0020), horizontal tabs (U+0009), carriage
returns (U+000D), and newlines (U+000A), is ignored except as it separates
tokens that would otherwise combine into a single token.

### Identifiers

Identifiers name program entities such as variables and types. An identifier is
a sequence of one or more letters and digits. The first character in an
identifier must be a letter.

```
identifier = letter { letter | unicode_digit } .
```

```
a
_x9
This_is_aValidIdentifier
αβ
```

### Keywords

The following keywords are reserved and may not be used as identifiers.

```
break
continue
else
false
fn
for
if
import
in
let
match
new
return
self
struct
true
while 
```

### Operators and Punctuation

The following character sequences represent operators (including assignment operators) and punctuation:

```
+
+=
&&
==
!=
(
)
-
-=
||
<
<=
[
]
*
*=
>
>=
{
}
/
/=
++
=
,
;
%
--
!
.
:
```

### Integer Literals

An integer literal is a sequence of digits representing an integer constant. An
optional prefix sets a non-decimal base: `0b` or `0B` for binary, `0`, `0o`, or
`0O` for octal, and `0x` or `0X` for hexadecimal. A single `0` is considered a
decimal zero. In hexadecimal literals, letters `a` through `f` and `A` through
`F` represent values `10` through `15`.

For readability, an underscore character `_` may appear after a base prefix or
between successive digits; such underscores do not change the literal's value.

```
int_lit        = decimal_lit | binary_lit | octal_lit | hex_lit .
decimal_lit    = "0" | ( "1" … "9" ) [ [ "_" ] decimal_digits ] .
binary_lit     = "0" ( "b" | "B" ) [ "_" ] binary_digits .
octal_lit      = "0" [ "o" | "O" ] [ "_" ] octal_digits .
hex_lit        = "0" ( "x" | "X" ) [ "_" ] hex_digits .

decimal_digits = decimal_digit { [ "_" ] decimal_digit } .
binary_digits  = binary_digit { [ "_" ] binary_digit } .
octal_digits   = octal_digit { [ "_" ] octal_digit } .
hex_digits     = hex_digit { [ "_" ] hex_digit } .

42
4_2
0600
0_600
0o600
0O600       // second character is capital letter 'O'
0xBadFace
0xBad_Face
0x_67_7a_2f_cc_40_c6
170141183460469231731687303715884105727
170_141183_460469_231731_687303_715884_105727

_42         // an identifier, not an integer literal
42_         // invalid: _ must separate successive digits
4__2        // invalid: only one _ at a time
0_xBadFace  // invalid: _ must separate successive digits
```
