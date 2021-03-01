# Specification

> **Note**: This specification is a work in progress.

## Introduction

This is a reference manual for the Antimony programming language.

Antimony is a general-purpose language designed with simplicity in mind. It is strongly typed and supports multiple compile-targets. Programs are constructed from modules, whose properties allow efficient management of dependencies.

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

Productions are expressions constructed from terms and the following operators, in increasing precedence:

```
|   alternation
()  grouping
[]  option (0 or 1 times)
{}  repetition (0 to n times)
```

Lower-case production names are used to identify lexical tokens. Non-terminals are in CamelCase. Lexical tokens are enclosed in double quotes `""` or single quotes `''`.

The form `a ... b` represents the set of characters from `a` through `b` as alternatives. The horizontal ellipsis `...` is also used elsewhere in the spec to informally denote various enumerations or code snippets that are not further specified. The character `…` (as opposed to the three characters `...`) is not a token of the Antimony language.

## Source Code Representation

Source code is Unicode text encoded in [UTF-8](https://en.wikipedia.org/wiki/UTF-8). The text is not canonicalized, so a single accented code point is distinct from the same character constructed from combining an accent and a letter; those are treated as two code points. For simplicity, this document will use the unqualified term _character_ to refer to a Unicode code point in the source text.

Each code point is distinct; for instance, upper and lower case letters are different characters.

Implementation restriction: For compatibility with other tools, a compiler may disallow the NUL character (U+0000) in the source text.

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

Comments serve as program documentation. A comment starts with the character sequence // and stop at the end of the line.

A comment cannot start inside a string literal, or inside a comment.

### Tokens

Tokens form the vocabulary of the Antimony programming language. There are four classes: _identifiers_, _keywords_, _operators and punctuation_, and _literals_. _White space_, formed from spaces (U+0020), horizontal tabs (U+0009), carriage returns (U+000D), and newlines (U+000A), is ignored except as it separates tokens that would otherwise combine into a single token.
