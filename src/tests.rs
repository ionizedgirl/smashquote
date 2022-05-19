use crate::*;
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
