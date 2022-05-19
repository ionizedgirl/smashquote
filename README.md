# smashquote

smashquote - Removes C-like quotes from byte slices

`smashquote` removes C-like quotes form byte slices. Specifically,
it understands the bash `$''` format. Unlike [snailquote](https://github.com/euank/snailquote),
smashquote works on byte slices. It is intended for use in command line
utilities and argument parsing where [OsString](std::ffi::OsString) handling may be desired,
rather than handling for unicode [String](std::string::String)s.

License: MIT OR Apache-2.0 OR GPL-3.0-or-later
