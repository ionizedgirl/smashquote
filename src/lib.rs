#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(missing_docs)]

//! smashquote - Removes C-like quotes from byte slices
//!
//! `smashquote` removes C-like quotes and escape sequences from byte slices. Specifically,
//! it understands the bash `$''` format. Unlike [snailquote](https://github.com/euank/snailquote),
//! smashquote works on byte slices. It is intended for use in command line
//! utilities and argument parsing where [OsString](std::ffi::OsString) handling may be desired,
//! rather than handling for unicode [String](std::string::String)s.
//! Thus, smashquote does not necessarily produce valid Unicode.
//!
//! For example, one may wish to have a CLI utility that takes a delimiter, such
//! as xargs or cut. In this situation, it's convienent for the user to enter
//! arguments like `-d '\r\n'` on the command line. smashquote can be used to
//! transform them into the correct sequence of bytes.
//!
//! smashquote understands the following backslash-escape sequences:
//! * `\a` - alert/bell `0x07`
//! * `\b` - backspace `0x08`
//! * `\e` - escape `0x1B`
//! * `\f` - form feed `0x0C`
//! * `\n` - line feed `0x0A` (unix newline)
//! * `\r` - carriage return `0x0D`
//! * `\t` - tab `0x09` (horizontal tab)
//! * `\v` - vertical tab `0x0B`
//! * `\\` - backslash `0x5C` (a single `\`)
//! * `\'` - single quote `0x27` (a single `'`)
//! * `\"` - double quote `0x22` (a single `"`)
//! * `\0` through `\377` - a single byte, specified in octal. The sequence stops at the first character that's not a hexidecimal digit.
//! * `\x0` through `\xFF` - a single byte, specified in hex. The sequence stops at the first character that's not a hexidecimal digit.
//! * `\u0` through `\uFFFF` - utf8 bytes of a single character, specified in hex. The sequence stops at the first character that's not a hexidecimal digit.
//! * `\u{0}` through `\u{10FFFF}` - utf8 bytes of a single character, specified in Rust style hex
//! * `\U0` through `\UFFFFFFFF` - utf8 bytes of a single character, specified in hex (of course, the actual maximum is 10FFFF, because that's currently the maximum valid codepoint). The sequence stops at the first character that's not a hexidecimal digit.
//! * `\c@`, `\cA` through `\cZ`, `\c[`, `\c\`, `\c]`, `\c^`, `\c_` - a control-x character (case insensitive, for some reason) `0x0` through `0x1F`
//! * ``\c` ``, `\ca` through `\cz`, `\c{`, `\c|`, `\c}`, `\c~` - a control-x character (same as above) `0x0` through `0x1F`


use std::iter::Peekable;
use std::io::Write;

/// Prints bytes as space-separated hex digits
pub fn pretty_bytes(bs: &[u8]) -> String {
    bs
        .iter()
        .map(|byte| format!("{:02X}", byte))
        .collect::<Vec<String>>()
        .join(" ")
}

/// Tries to represent bytes as presentable unicode
pub fn pretty_string(bs: &[u8]) -> String {
    String::from_utf8_lossy(bs).chars()
        .map(|c| match c {
        '\u{0}'..='\u{20}' => char::from_u32((c as u32) + 0x2400u32).unwrap(),
        '\u{7F}' => '\u{247F}',
        _ => c,
        }).collect()
}

/// Describes what kind of invalid backslash escape was found
#[derive(Debug)]
pub enum InvalidBackslashKind {
    /// `\u{`
    RustStyleUnicodeMissingCloseBrace,
    /// `\u{}`
    RustStyleUnicodeMissingDigits,
    /// Unicode escape wasn't a valid codepoint
    UnicodeEscapeBadCodepoint,
    /// Non-unicode bytes instead of hex digits
    HexDigitsNotUnicode,
    /// Hex digits werent 0-9A-Fa-f
    HexDigitsNotHexDigits(Vec<u8>),
    /// `\x` not followed by any digits
    HexDigitsNoDigits,
    /// Non-unicode bytes instead of octal digits
    OctalDigitsNotUnicode,
    /// Octal digits weren't 0-7
    OctalDigitsNotOctalDigits,
    /// `\u` not followed by any digits
    UnicodeEscapeNoDigits,
    /// `\u` right at end of the string
    UnicodeEscapeEndOfString,
    /// `\c` followed by an unknown key
    ControlEscapeBadKey,
    /// `\c` right at the end of the string
    ControlEscapeEndOfString,
    /// `\` followed by an unknown character
    BackslashEscapeUnknown,
    /// `\` right at the end of the string
    BackslashEndOfString,
}

use InvalidBackslashKind::*;

/// Error type of unescape/unquote functions.
#[derive(Debug)]
pub enum UnescapeError {
    /// An invalid backslash escape sequence while parsing
    InvalidBackslash {
        /// Which kind of invalid backslash, where parsing failed
        kind: InvalidBackslashKind,
        
        /// The byte offset of the backslash escape
        offset: usize,
        
        /// An attempt at showing the backslash escape sequence as a string
        string: String,
        
        /// The backslash escape sequence as raw hex bytes
        bytes: String,
    },
    
    /// Reached end of string while looking for closing delimiter byte
    MissingClose {
        /// An attempt at showing the close delimiter
        string: String,
        
        /// The close delimiter as raw hex bytes
        bytes: String,
    },
    
    /// Some I/O error happened...
    IOError(std::io::Error),
}

impl UnescapeError {
    /// Generates a [MissingClose](UnescapeError::MissingClose) error from a 1-byte delimiter
    pub fn missing_close(byte: u8) -> Self {
        return Self::MissingClose {
            string: pretty_string(&[byte]),
            bytes: pretty_bytes(&[byte]),
        };
    }
    
    /// Generates an [InvalidBackslash](UnescapeError::InvalidBackslash) error
    pub fn invalid_backslash(
        offset: usize,
        bytes: &[u8],
        kind: InvalidBackslashKind,
    ) -> Self {
        return Self::InvalidBackslash {
            kind: kind,
            offset: offset,
            string: pretty_string(bytes),
            bytes: pretty_bytes(bytes),
        }
    }
}

impl From<std::io::Error> for UnescapeError {
    fn from(error: std::io::Error) -> Self {
        UnescapeError::IOError(error)
    }
}

fn unhex<'a>(
    offset: usize,
    escape: &[u8],
    start: usize,
    end: Option<usize>,
) -> Result<Vec<u8>, UnescapeError>
{
    let range = match end {
        Some(i) => escape[start..=i].to_vec(),
        None => escape[start..].to_vec(),
    };
    let hex: String = match String::from_utf8(range.clone()) {
        Ok(s) => s,
        Err(_) => { return Err(UnescapeError::invalid_backslash(offset, &escape, HexDigitsNotUnicode)); }
    };
    let ord: u32 = match u32::from_str_radix(&hex, 16) {
        Ok(b) => b,
        Err(_) => { return Err(UnescapeError::invalid_backslash(offset, &escape, HexDigitsNotHexDigits(range))); }
    };
    let out_char: char = match char::from_u32(ord) {
        Some(c) => c,
        None => {
            return Err(UnescapeError::invalid_backslash(offset, &escape, UnicodeEscapeBadCodepoint));
        }
    };
    let mut s = String::with_capacity(8);
    s.push(out_char);
    return Ok(s.into_bytes());
}

fn un_rust_style_u<'a, I>(
    bytes: &mut Peekable<I>,
    offset: usize,
    escape: &mut Vec<u8>,
) -> Result<Vec<u8>, UnescapeError>
where
    I: Iterator<Item = (usize, &'a u8)>,
    I: ExactSizeIterator<Item = (usize, &'a u8)>,
{
    let mut found_close = false;
    while let Some((_, &byte4)) = bytes.next() {
        escape.push(byte4);
        if byte4 == b'}' {
            found_close = true;
            break;
        }
    }
    if ! found_close {
        return Err(UnescapeError::invalid_backslash(offset, &escape, RustStyleUnicodeMissingCloseBrace));
    }
    let end = escape.len()-2;
    let start = 3;
    if end == start-1 {
        return Err(UnescapeError::invalid_backslash(offset, &escape, RustStyleUnicodeMissingDigits));
    } else if end < start {
        unreachable!();
    }
    return Ok(unhex(offset, &escape, start, Some(end))?);
}


/// Writes an unescaped string from an iterator
/// 
/// # Arguments
/// 
/// * `bytes` - An iterator that yields a position and byte like `[u8].iter().enumerate().peekable()`
/// * `out` - An output stream, like `Vec<u8>`
/// * `close` - An optional closing delimiter to look for
pub fn unescape_iter<'a, I, O>(
    bytes: &mut Peekable<I>, 
    out: &mut O, 
    close: Option<u8>
) -> Result<usize, UnescapeError>
where
    I: Iterator<Item = (usize, &'a u8)>,
    I: ExactSizeIterator<Item = (usize, &'a u8)>,
    O: Write,
{
    // This is a workaround for https://github.com/rust-lang/rust/issues/53667
    let close_delimiter: u8;
    let have_close: bool;
    match close {
        Some(b) => {
            close_delimiter = b;
            have_close = true;
        }
        None => {
            close_delimiter = 0;
            have_close = false;
        }
    }
    
    let mut last_offset: Option<usize> = None;
    
    while let Some((offset, &byte)) = bytes.next() {
        if byte == b'\\' {
            let mut escape: Vec<u8> = Vec::with_capacity(12);
            escape.push(byte);
            if let Some((_, &byte2)) = bytes.next() {
                escape.push(byte2);
                let _wrote = match byte2 {
                    b'a' => out.write(&[0x07])?, // alert/bell
                    b'b' => out.write(&[0x08])?, // backspace
                    b'e' | b'E' => out.write(&[0x1B])?, // escape
                    b'f' => out.write(&[0x0C])?, // form feed
                    b'n' => out.write(&[0x0A])?, // newline or line feed
                    b'r' => out.write(&[0x0D])?, // carriage return
                    b't' => out.write(&[0x09])?, // horizontal tab
                    b'v' => out.write(&[0x0B])?, // vertical tab
                    b'\'' => out.write(&[b'\''])?, // single quote
                    b'"' => out.write(&[b'"'])?, // double quote
                    b'\\' => out.write(&[b'\\'])?, // literal backslash
                    b'0'..=b'9' => {
                        for _ in 3..=4 {
                            if let Some((_, &byte3)) = bytes.peek() {
                                if byte3.is_ascii_digit() {
                                    escape.push(byte3);
                                }
                                let (_, _) = bytes.next().unwrap();
                            }
                        }
                        let octal: String = match String::from_utf8(escape[1..].to_vec()) {
                            Ok(s) => s,
                            Err(_) => { return Err(UnescapeError::invalid_backslash(offset, &escape, OctalDigitsNotUnicode)); }
                        };
                        let out_byte: u8 = match u8::from_str_radix(&octal, 8) {
                            Ok(b) => b,
                            Err(_) => { return Err(UnescapeError::invalid_backslash(offset, &escape, OctalDigitsNotOctalDigits)); }
                        };
                        out.write(&[out_byte])?
                    }
                    b'x' => { // this one could be bad unicode, its a byte
                        for _ in 3..=4 {
                            if let Some((_, &byte3)) = bytes.peek() {
                                if byte3.is_ascii_hexdigit() {
                                    escape.push(byte3);
                                }
                                let (_, _) = bytes.next().unwrap();
                            }
                        }
                        if escape.len() == 2 { // just \x
                            return Err(UnescapeError::invalid_backslash(offset, &escape, HexDigitsNoDigits));
                        }
                        let hex: String = match String::from_utf8(escape[2..].to_vec()) {
                            Ok(s) => s,
                            Err(_) => { return Err(UnescapeError::invalid_backslash(offset, &escape, HexDigitsNotUnicode)); }
                        };
                        let out_byte: u8 = match u8::from_str_radix(&hex, 16) {
                            Ok(b) => b,
                            Err(_) => { return Err(UnescapeError::invalid_backslash(offset, &escape, HexDigitsNotHexDigits(hex.as_bytes().to_vec()))); }
                        };
                        out.write(&[out_byte])?
                    }
                    b'u' => {
                        if let Some((_, &byte3)) = bytes.next() {
                            escape.push(byte3);
                            if byte3 == b'{' {
                                let u_bytes: Vec<u8> = un_rust_style_u(bytes, offset, &mut escape)?;
                                out.write(&u_bytes.as_slice())?
                            } else {
                                if ! byte3.is_ascii_hexdigit() {
                                    return Err(UnescapeError::invalid_backslash(offset, &escape, UnicodeEscapeNoDigits));
                                }
                                for _ in 4..=6 {
                                    if let Some((_, &byte4)) = bytes.peek() {
                                        if byte3.is_ascii_hexdigit() {
                                            escape.push(byte4);
                                        }
                                        let (_, _) = bytes.next().unwrap();
                                    }
                                }
                                let utf8 = unhex(offset, &escape, 2, None)?;
                                out.write(&utf8.as_slice())?
                            }
                        } else {
                            return Err(UnescapeError::invalid_backslash(offset, &escape, UnicodeEscapeEndOfString));
                        }
                    }
                    b'U' => {
                        if let Some((_, &byte3)) = bytes.next() {
                            escape.push(byte3);
                            if ! byte3.is_ascii_hexdigit() {
                                return Err(UnescapeError::invalid_backslash(offset, &escape, UnicodeEscapeNoDigits));
                            }
                            for _ in 4..=10 {
                                if let Some((_, &byte4)) = bytes.peek() {
                                    if byte3.is_ascii_hexdigit() {
                                        escape.push(byte4);
                                    }
                                    let (_, _) = bytes.next().unwrap();
                                }
                            }
                            let utf8 = unhex(offset, &escape, 2, None)?;
                            out.write(&utf8.as_slice())?
                        } else {
                            return Err(UnescapeError::invalid_backslash(offset, &escape, UnicodeEscapeEndOfString));
                        }
                    }
                    b'c' => {
                        if let Some((_, &byte3)) = bytes.next() {
                            escape.push(byte3);
                            if (b'@'..=b'_').contains(&byte3) {
                                out.write(&[byte3-0x40].as_slice())?
                            } else if (b'`'..=b'~').contains(&byte3) {
                                out.write(&[byte3-0x60].as_slice())?
                            } else {
                                return Err(UnescapeError::invalid_backslash(offset, &escape, ControlEscapeBadKey));
                            }
                        } else {
                            return Err(UnescapeError::invalid_backslash(offset, &escape, ControlEscapeEndOfString));
                        }
                    }
                    _ => { return Err(UnescapeError::invalid_backslash(offset, &escape, BackslashEscapeUnknown)); }
                };
            } else {
                UnescapeError::invalid_backslash(offset, &escape, BackslashEndOfString);
            }
        } else if have_close && byte == close_delimiter {
            return Ok(offset);
        } else {
            out.write(&[byte])?;
        }
        last_offset = Some(offset);
    }
    
    // At this point we have run out of bytes!
    
    if have_close {
        Err(UnescapeError::missing_close(close_delimiter))
    } else {
        return Ok(last_offset.unwrap());
    }
}

/// Returns a new unescaped byte string from a byte slice
/// 
/// # Arguments
/// 
/// * `bytes` - A slice of bytes
pub fn unescape_bytes(
    bytes: &[u8],
) -> Result<Vec<u8>, UnescapeError> {
    let mut r: Vec<u8> = Vec::with_capacity(bytes.len());
    unescape_iter(&mut bytes.iter().enumerate().peekable(), &mut r, None)?;
    return Ok(r);
}

#[cfg(test)]
mod tests;

