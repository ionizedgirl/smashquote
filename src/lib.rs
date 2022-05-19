#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(missing_docs)]

//! smashquote - Removes shell-like quotes from byte slices

use thiserror::Error;
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

/// Error type of [unescape](unescape).
#[derive(Debug, Error)]
pub enum UnescapeError {
    #[error("Invalid backslash-escape {string} at byte {offset}: {bytes}")]
    /// An invalid backslash escape sequence while parsing
    InvalidBackslash {
        /// The byte offset of the backslash escape
        offset: usize,
        
        /// An attempt at showing the backslash escape sequence as a string
        string: String,
        
        /// The backslash escape sequence as raw hex bytes
        bytes: String,
    },
    
    #[error("Reached end of string while looking for closing {string} ({bytes})")]
    /// Reached end of string while looking for closing delimiter byte
    MissingClose {
        /// An attempt at showing the close delimiter
        string: String,
        
        /// The close delimiter as raw hex bytes
        bytes: String,
    },
    
    #[error("I/O error {0}")]
    /// Some I/O error happened...
    IOError(std::io::Error),
}

impl UnescapeError {
    /// Generates a [MissingClose](MissingClose) error from a 1-byte delimiter
    pub fn missing_close(byte: u8) -> Self {
        return Self::MissingClose {
            string: pretty_string(&[byte]),
            bytes: pretty_bytes(&[byte]),
        };
    }
    /// Generates an [InvalidBackslash](InvalidBackslash) error
    pub fn invalid_backslash(offset: usize, bytes: &[u8]) -> Self {
        return Self::InvalidBackslash {
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

fn unhex(escape: &[u8]) -> Option<Vec<u8>> {
    let hex: String = match String::from_utf8(escape.to_vec()) {
        Ok(s) => s,
        Err(_) => { return None; }
    };
    let ord: u32 = match u32::from_str_radix(&hex, 16) {
        Ok(b) => b,
        Err(_) => { return None; }
    };
    let out_char: char = match char::from_u32(ord) {
        Some(c) => c,
        None => { return None; }
    };
    let mut s = String::with_capacity(8);
    s.push(out_char);
    return Some(s.into_bytes());
}

fn un_rust_style_u<'a, I>(
    bytes: &mut I,
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
        if byte4 == "}" {
            found_close = true;
            break;
        }
    }
    if ! found_close {
        return Err(UnescapeError::invalid_backslash(offset, &escape));
    }
    let end = escape.len()-2;
    let start = 3;
    if end == start {
        return Err(UnescapeError::invalid_backslash(offset, &escape));
    } else if end < start {
        unreachable!();
    }
    
    if let Some(utf8) = unhex(&escape[start..=end]) {
        return Ok(utf8);
    } else {
        return Err(UnescapeError::invalid_backslash(offset, &escape));
    }
}


/// Writes an unescaped string from an iterator
/// 
/// # Arguments
/// 
/// * `bytes` - An iterator that yields a position and byte like `[u8].iter().enumerate()`
/// * `out` - An output stream, like `Vec<u8>`
/// * `close` - An optional closing delimiter to look for
pub fn unescape_iter<'a, I, O>(
    bytes: &mut I, 
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
                    b'0' | b'1' => {
                        if let Some((_, &byte3)) = bytes.next() {
                            escape.push(byte3);
                        } else {
                            return Err(UnescapeError::invalid_backslash(offset, &escape));
                        }
                        if let Some((_, &byte4)) = bytes.next() {
                            escape.push(byte4);
                        } else {
                            return Err(UnescapeError::invalid_backslash(offset, &escape));
                        }
                        let octal: String = match String::from_utf8(escape[1..].to_vec()) {
                            Ok(s) => s,
                            Err(_) => { return Err(UnescapeError::invalid_backslash(offset, &escape)); }
                        };
                        let out_byte: u8 = match u8::from_str_radix(&octal, 8) {
                            Ok(b) => b,
                            Err(_) => { return Err(UnescapeError::invalid_backslash(offset, &escape)); }
                        };
                        out.write(&[out_byte])?
                    }
                    b'x' => {
                        if let Some((_, &byte3)) = bytes.next() {
                            escape.push(byte3);
                        } else {
                            return Err(UnescapeError::invalid_backslash(offset, &escape));
                        }
                        if let Some((_, &byte4)) = bytes.next() {
                            escape.push(byte4);
                        } else {
                            return Err(UnescapeError::invalid_backslash(offset, &escape));
                        }
                        if let Some(utf8) = unhex(&escape[1..]) {
                            out.write(&utf8.as_slice())?
                        } else {
                            return Err(UnescapeError::invalid_backslash(offset, &escape));
                        }
                    }
                    b'u' => {
                        if let Some((_, &byte3)) = bytes.next() {
                            escape.push(byte3);
                            if byte3 == b'{' {
                            } else {
                            }
                        } else {
                            UnescapeError::invalid_backslash(offset, &escape);
                        }
                        out.write(&[out_byte])?
                    }
                    _ => { return Err(UnescapeError::invalid_backslash(offset, &escape)); }
                };
            } else {
                UnescapeError::invalid_backslash(offset, &escape);
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
