pub fn is_identifier(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

pub fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}

pub fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_identifier() {
        assert_eq!(is_identifier('a'), true);
        assert_eq!(is_identifier('_'), true);
        assert_eq!(is_identifier('あ'), false);
        assert_eq!(is_identifier('1'), false);
    }

    #[test]
    fn test_is_digit() {
        assert_eq!(is_digit('1'), true);
        assert_eq!(is_digit('2'), true);
        assert_eq!(is_digit('3'), true);
        assert_eq!(is_digit('4'), true);
        assert_eq!(is_digit('_'), false);
        assert_eq!(is_digit('%'), false);
        assert_eq!(is_digit('１'), false); // non ascii 1
    }

    #[test]
    fn test_is_whitespace() {
        assert_eq!(is_whitespace(' '), true);
        assert_eq!(is_whitespace('\n'), true);
        assert_eq!(is_whitespace('\t'), true);
        assert_eq!(is_whitespace('　'), true); // non-ascii space
    }
}