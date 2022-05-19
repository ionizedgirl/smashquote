# smashquote

smashquote - Removes C-like quotes from byte slices

`smashquote` removes C-like quotes form byte slices. Specifically,
it understands the bash `$''` format. Unlike [snailquote](https://github.com/euank/snailquote),
smashquote works on byte slices. It is intended for use in command line
utilities and argument parsing where [OsString](std::ffi::OsString) handling may be desired,
rather than handling for unicode [String](std::string::String)s.
Thus, smashquote does not necessarily produce valid Unicode.

For example, one may wish to have a CLI utility that takes a delimiter, such
as xargs or cut. In this situation, it's convienent for the user to enter
arguments like `-d '\r\n'` on the command line. smashquote can be used to
transform them into the correct sequence of bytes.

smashquote understands the following backslash-escape sequences:
* `\a` - alert/bell `0x07`
* `\b` - backspace `0x08`
* `\e` - escape `0x1B`
* `\f` - form feed `0x0C`
* `\n` - line feed `0x0A` (unix newline)
* `\r` - carriage return `0x0D`
* `\t` - tab `0x09` (horizontal tab)
* `\a` - vertical tab `0x0B`
* `\\` - backslash `0x5C` (a single `\`)
* `\'` - single quote `0x27` (a single `'`)
* `\"` - double quote `0x22` (a single `"`)
* `\0` through `\377` - a single byte, specified in octal. The sequence stops at the first character that's not a hexidecimal digit.
* `\x0` through `\xFF` - a single byte, specified in hex. The sequence stops at the first character that's not a hexidecimal digit.
* `\u0` through `\uFFFF` - utf8 bytes of a single character, specified in hex. The sequence stops at the first character that's not a hexidecimal digit.
* `\u{0}` through `\u{10FFFF}` - utf8 bytes of a single character, specified in Rust style hex
* `\U0` through `\UFFFFFFFF` - utf8 bytes of a single character, specified in hex (of course, the actual maximum is 10FFFF, because that's currently the maximum valid codepoint). The sequence stops at the first character that's not a hexidecimal digit.
* `\c@`, `\cA` through `\cZ`, `\c[`, `\c\`, `\c]`, `\c^`, `\c_` - a control-x character (case insensitive, for some reason) `0x0` through `0x1F`

License: MIT OR Apache-2.0 OR GPL-3.0-or-later
