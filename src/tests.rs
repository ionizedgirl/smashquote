use crate::*;
use anyhow;

#[test]
fn alarm() {
    let r = unescape_bytes(&b"\\a".as_slice()).unwrap();
    assert_eq!(r, [7]);
}
#[test]
fn backspace() {
    let r = unescape_bytes(&b"\\b".as_slice()).unwrap();
    assert_eq!(r, [8]);
}
#[test]
fn escape() {
    let r = unescape_bytes(&b"\\e\\E".as_slice()).unwrap();
    assert_eq!(r, [27, 27]);
}
#[test]
fn form_feed() {
    let r = unescape_bytes(&b"\\f".as_slice()).unwrap();
    assert_eq!(r, [12]);
}
#[test]
fn line_feed() {
    let r = unescape_bytes(&b"\\n".as_slice()).unwrap();
    assert_eq!(r, [10]);
}
#[test]
fn carriage_return() {
    let r = unescape_bytes(&b"\\r".as_slice()).unwrap();
    assert_eq!(r, [13]);
}
#[test]
fn tab() {
    let r = unescape_bytes(&b"\\t".as_slice()).unwrap();
    assert_eq!(r, [9]);
}
#[test]
fn vertical_tab() {
    let r = unescape_bytes(&b"\\v".as_slice()).unwrap();
    assert_eq!(r, [11]);
}
#[test]
fn backslash() {
    let r = unescape_bytes(&b"\\\\".as_slice()).unwrap();
    assert_eq!(r, b"\\");
}
#[test]
fn single_quote() {
    let r = unescape_bytes(&b"\\'".as_slice()).unwrap();
    assert_eq!(r, b"'");
}
#[test]
fn double_quote() {
    let r = unescape_bytes(&b"\\\"".as_slice()).unwrap();
    assert_eq!(r, b"\"");
}
#[test]
fn null() {
    let r = unescape_bytes(&b"\\0".as_slice()).unwrap();
    assert_eq!(r, [0]);
}
#[test]
fn octal() {
    for i in 0..=255 {
        let s = format!("\\{i:o}");
        let r = unescape_bytes(&s.as_bytes()).unwrap();
        assert_eq!(r, [i]);
    }
}
#[test]
fn octal0() {
    for i in 0..=255 {
        let s = format!("\\{i:03o}");
        let r = unescape_bytes(&s.as_bytes()).unwrap();
        assert_eq!(r, [i]);
    }
}
#[test]
fn hex() {
    for i in 0..=255 {
        let s = format!("\\x{i:x}");
        let r = unescape_bytes(&s.as_bytes()).unwrap();
        assert_eq!(r, [i]);
    }
}
#[test]
fn hex0() {
    for i in 0..=255 {
        let s = format!("\\x{i:02x}");
        let r = unescape_bytes(&s.as_bytes()).unwrap();
        assert_eq!(r, [i]);
    }
}
#[test]
fn unicode4() {
    for i in 0u32..=0xFFFF {
        match char::from_u32(i) {
            Some(c) => {
                let s = format!("\\u{i:x}");
                let r = unescape_bytes(&s.as_bytes()).unwrap();
                let mut s2 = String::with_capacity(8);
                s2.push(c);
                assert_eq!(r, s2.as_bytes());
            }
            None => {
                let s = format!("\\u{i:x}");
                let r = unescape_bytes(&s.as_bytes());
                assert!(r.is_err());
            }
        }
    }
}
#[test]
fn unicode04() {
    for i in 0u32..=0xFFFF {
        match char::from_u32(i) {
            Some(c) => {
                let s = format!("\\u{i:04x}");
                let r = unescape_bytes(&s.as_bytes()).unwrap();
                let mut s2 = String::with_capacity(8);
                s2.push(c);
                assert_eq!(r, s2.as_bytes());
            }
            None => {
                let s = format!("\\u{i:04x}");
                let r = unescape_bytes(&s.as_bytes());
                assert!(r.is_err());
            }
        }
    }
}
#[test]
fn unicode_rust_style() {
    for i in 0u32..=0x10FFFF {
        match char::from_u32(i) {
            Some(c) => {
                let s = format!("\\u{{{i:x}}}");
                let r = unescape_bytes(&s.as_bytes()).unwrap();
                let mut s2 = String::with_capacity(8);
                s2.push(c);
                assert_eq!(r, s2.as_bytes());
            }
            None => {
                let s = format!("\\u{i:04x}");
                let r = unescape_bytes(&s.as_bytes());
                assert!(r.is_err());
            }
        }
    }
}
#[test]
fn unicode8() {
    for i in 0u32..=0x10FFFF {
        match char::from_u32(i) {
            Some(c) => {
                let s = format!("\\U{i:x}");
                let r = unescape_bytes(&s.as_bytes()).unwrap();
                let mut s2 = String::with_capacity(8);
                s2.push(c);
                assert_eq!(r, s2.as_bytes());
            }
            None => {
                let s = format!("\\u{i:x}");
                let r = unescape_bytes(&s.as_bytes());
                assert!(r.is_err());
            }
        }
    }
}
#[test]
fn unicode08() {
    for i in 0u32..=0x10FFFF {
        match char::from_u32(i) {
            Some(c) => {
                let s = format!("\\U{i:08x}");
                let r = unescape_bytes(&s.as_bytes()).unwrap();
                let mut s2 = String::with_capacity(8);
                s2.push(c);
                assert_eq!(r, s2.as_bytes());
            }
            None => {
                let s = format!("\\u{i:04x}");
                let r = unescape_bytes(&s.as_bytes());
                assert!(r.is_err());
            }
        }
    }
}
#[test]
fn control_x() {
    for x in b'@'..=b'~' {
        let c = x & 0x1F;
        let mut b = Vec::with_capacity(10);
        b.extend(b"\\c");
        b.push(x);
        let r = unescape_bytes(&b).unwrap();
        assert_eq!(r, &[c]);
    }
}
#[test]
fn anyhow_compatible() {
    let _unescape_error = anyhow::Error::new::<UnescapeError>(UnescapeError::InvalidBackslash {
        kind: InvalidBackslashKind::RustStyleUnicodeMissingCloseBrace,
        string: String::new(),
        bytes: String::new(),
        offset: 0,
    });
}
