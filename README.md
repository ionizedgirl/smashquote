# smashquote

smashquote - Removes C-like quotes from byte slices

`smashquote` removes C-like quotes form byte slices. Specifically,
it understands the bash `$''` format. Unlike [snailquote](https://github.com/euank/snailquote),
smashquote works on byte slices. It is intended for use in command line
utilities and argument parsing where [OsString](std::ffi::OsString) handling may be desired,
rather than handling for unicode [String](std::string::String)s.
Thus, smashquote does not necessarily produce valid Unicode.

smashquote understands the following backslash-escape sequences:
* `\a` - alert/bell `0x07`
* `\b` - backspace `0x08`
* `\e` - escape `0x1B`
* `\f` - form feed `0x0C`
* `\n` - line feed `0x0A` (unix newline)
* `\r` - carriage return `0x0D`
* `\t` - tab `0x09` (horizontal tab)
* `\a` - vertical tab (0x0B)
* `\\` - backslash (0x5C) (a single `\`)
* `\'` - single quote (0x27) (a single `'`)
* `\"` - double quote (0x22) (a single `"`)
* `\0` through `\377` - a single byte, specified in octal
* `\x0` through `\xFF` - a single byte, specified in hex
* `\u0` through `\uFFFF` - utf8 bytes of a single character, specified in hex
* `\u{0}` through `\u{10FFFF}` - utf8 bytes of a single character, specified in Rust style hex
* `\U0` through `\UFFFFFFFF` - utf8 bytes of a single character, specified in hex (of course the actual maximum is 10FFFF)

License: MIT OR Apache-2.0 OR GPL-3.0-or-later
