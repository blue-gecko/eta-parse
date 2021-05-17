use std::{
    borrow::Cow,
    cmp::Ordering,
    convert::{From, TryFrom},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Align {
    Left,
    Right,
}

impl TryFrom<&str> for Align {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from(s.to_string())
    }
}

impl TryFrom<String> for Align {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().trim() {
            "left" => Ok(Align::Left),
            "right" => Ok(Align::Right),
            _ => Err(String::from("Unknown align argument")),
        }
    }
}

#[allow(dead_code)]
pub fn truncate(s: &str, width: usize) -> Cow<str> {
    _truncate(s, width, s.chars().count())
}

pub fn _truncate(s: &str, width: usize, len: usize) -> Cow<str> {
    if len > width {
        s[..width].into()
    } else {
        s.into()
    }
}

#[allow(dead_code)]
pub fn pad(s: &str, width: usize, align: Align, padding: char) -> Cow<str> {
    _pad(s, width, align, padding, s.chars().count())
}

#[allow(dead_code)]
pub fn _pad(s: &str, width: usize, align: Align, padding: char, len: usize) -> Cow<str> {
    if len < width {
        let mut buf = String::with_capacity(width);
        buf.push_str(s);
        match align {
            Align::Left => {
                for _ in len..width {
                    buf.push(padding);
                }
                buf.into()
            }
            Align::Right => {
                for _ in len..width {
                    buf.insert(0, padding);
                }
                buf.into()
            }
        }
    } else {
        s.into()
    }
}

#[allow(dead_code)]
pub fn fixed_width(s: &str, width: usize, align: Align, padding: char) -> Cow<str> {
    let len = s.chars().count();
    match width.cmp(&len) {
        Ordering::Less => _truncate(s, width, len),
        Ordering::Greater => _pad(s, width, align, padding, len),
        _ => s.into(),
    }
}

#[allow(dead_code)]
pub fn strip_padding(s: &str, align: Align, padding: char) -> Cow<str> {
    match align {
        Align::Left => {
            if s.ends_with(padding) {
                let mut end = s.len();
                for (i, c) in s.char_indices().rev() {
                    if c == padding {
                        end = i;
                    } else {
                        break;
                    }
                }
                s.get(0..end).expect("Unexpected out of bounds").into()
            } else {
                s.into()
            }
        }
        Align::Right => {
            if s.starts_with(padding) {
                s.chars()
                    .skip_while(|c| *c == padding)
                    .collect::<String>()
                    .into()
            } else {
                s.into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_try_from_str() {
        assert_eq!(Align::try_from("LEFT"), Ok(Align::Left));
        assert_eq!(Align::try_from("Right"), Ok(Align::Right));
        assert!(matches!(Align::try_from("Banana"), Err(_)));
    }

    #[test]
    fn align_try_from_string() {
        assert_eq!(Align::try_from("LEFT".to_string()), Ok(Align::Left));
        assert_eq!(Align::try_from("Right".to_string()), Ok(Align::Right));
        assert!(matches!(Align::try_from("Banana".to_string()), Err(_)));
    }

    #[test]
    fn truncate_shorter() {
        assert_eq!(truncate("1234567890", 5), "12345".to_string())
    }

    #[test]
    fn truncate_exact() {
        assert_eq!(truncate("1234567890", 10), "1234567890".to_string())
    }

    #[test]
    fn truncate_longer() {
        assert_eq!(truncate("1234567890", 15), "1234567890".to_string())
    }

    #[test]
    fn pad_left_shorter() {
        assert_eq!(
            pad("1234567890", 5, Align::Left, 'X'),
            "1234567890".to_string()
        )
    }

    #[test]
    fn pad_right_shorter() {
        assert_eq!(
            pad("1234567890", 5, Align::Right, '0'),
            "1234567890".to_string()
        )
    }

    #[test]
    fn pad_left_exact() {
        assert_eq!(
            pad("1234567890", 10, Align::Left, 'X'),
            "1234567890".to_string()
        )
    }

    #[test]
    fn pad_right_exact() {
        assert_eq!(
            pad("1234567890", 10, Align::Right, '0'),
            "1234567890".to_string()
        )
    }
    #[test]
    fn pad_left_longer() {
        assert_eq!(
            pad("1234567890", 15, Align::Left, 'X'),
            "1234567890XXXXX".to_string()
        )
    }

    #[test]
    fn pad_right_longer() {
        assert_eq!(
            pad("1234567890", 15, Align::Right, '0'),
            "000001234567890".to_string()
        )
    }

    #[test]
    fn fixed_width_shorter() {
        assert_eq!(
            fixed_width("1234567890", 5, Align::Right, '0'),
            "12345".to_string()
        )
    }

    #[test]
    fn fixed_width_exact() {
        assert_eq!(
            fixed_width("1234567890", 10, Align::Right, '0'),
            "1234567890".to_string()
        )
    }

    #[test]
    fn fixed_width_left_longer() {
        assert_eq!(
            fixed_width("1234567890", 15, Align::Left, 'X'),
            "1234567890XXXXX".to_string()
        )
    }

    #[test]
    fn fixed_width_right_longer() {
        assert_eq!(
            fixed_width("1234567890", 15, Align::Right, '0'),
            "000001234567890".to_string()
        )
    }

    #[test]
    fn strip_padding_left() {
        assert_eq!(
            strip_padding("000ABCX0987XXX", Align::Left, 'X'),
            "000ABCX0987"
        );
    }

    #[test]
    fn strip_padding_left_none() {
        assert_eq!(
            strip_padding("000ABCX0987", Align::Left, 'X'),
            "000ABCX0987"
        );
    }

    #[test]
    fn strip_padding_right() {
        assert_eq!(
            strip_padding("000ABCX0987XXX", Align::Right, '0'),
            "ABCX0987XXX"
        );
    }

    #[test]
    fn strip_padding_right_none() {
        assert_eq!(
            strip_padding("ABCX0987XXX", Align::Right, '0'),
            "ABCX0987XXX"
        );
    }
}
