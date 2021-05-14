use std::collections::HashMap;
use std::convert::{From, Into, TryFrom, TryInto};
use std::fmt::Debug;
use std::ops::Range;
use std::str::Chars;

mod builder;
mod read;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LineBreak {
    None,
    NewLine,
    CrLf,
}

#[derive(Debug)]
pub struct Parser<'a> {
    fields: Vec<Field<'a>>,
    width: usize,
}

impl<'a> Parser<'a> {
    fn parse<T: Into<String>>(&self, s: T) -> HashMap<String, String> {
        let s: String = s.into();
        self._parse(&mut s.chars().by_ref())
    }

    fn _parse(&self, chars: &mut Chars) -> HashMap<String, String> {
        match chars.size_hint() {
            (_, Some(max)) if max >= self.width => (),
            _ => panic!("Insufficient buffer allocated"),
        }

        let mut map = HashMap::new();
        for field in &self.fields {
            field.parse(&mut map, chars);
        }
        map
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Align {
    Left,
    Right,
}

impl TryFrom<&str> for Align {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().trim() {
            "left" => Ok(Align::Left),
            "right" => Ok(Align::Right),
            _ => Err(String::from("Unknown align argument")),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Field<'a> {
    name: Option<&'a str>,
    width: u32,
    align: Align,
    padding: char,
}

#[allow(dead_code)]
impl<'a> Field<'a> {
    fn new(name: Option<&'a str>, width: u32, align: Align, padding: char) -> Self {
        Field {
            name,
            width,
            align,
            padding,
        }
    }

    pub fn with_name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn without_name(mut self) -> Self {
        self.name = None;
        self
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn with_range(mut self, range: Range<u32>) -> Self {
        self.width = range.end - range.start;
        self
    }

    pub fn with_align<T: TryInto<Align>>(mut self, align: T) -> Self {
        match align.try_into() {
            Ok(align) => self.align = align,
            Err(_) => eprintln!("Unable to parse argument as Align"),
        }
        self
    }

    pub fn with_padding<T: Into<char>>(mut self, padding: T) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn name(&self) -> Option<&str> {
        self.name
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn align(&self) -> Align {
        self.align
    }

    pub fn padding(&self) -> char {
        self.padding
    }

    fn parse(&self, map: &mut HashMap<String, String>, chars: &mut Chars) {
        let width = self.width() as usize;
        if let Some(name) = self.name {
            map.entry(name.to_string())
                .or_insert_with(|| chars.take(width).collect::<String>());
        } else {
            chars.take(width).for_each(|_| {});
        }
    }
}

impl<'a> Default for Field<'a> {
    fn default() -> Self {
        Self {
            name: None,
            width: 0,
            align: Align::Left,
            padding: ' ',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_align_into() {
        assert_eq!(Align::try_from("LEFT"), Ok(Align::Left));
        assert_eq!(Align::try_from("Right"), Ok(Align::Right));
        assert!(matches!(Align::try_from("Banana"), Err(_)));
    }

    #[test]
    fn check_parser() {
        let parser = Parser {
            fields: Vec::new(),
            width: 0,
        };

        assert_eq!(parser.fields.len(), 0);
    }

    #[test]
    fn check_parsing() {
        let fields = vec![Field::default().with_name("test").with_range(0..10)];
        let parser = Parser { fields, width: 10 };
        let map = parser.parse("1234567890");

        assert!(map.contains_key("test"));
        assert_eq!(map.get("test"), Some(&String::from("1234567890")));
    }

    #[test]
    fn check_parsing_two_fields() {
        let fields = vec![
            Field::default().with_name("test-1").with_range(0..5),
            Field::default().with_name("test-2").with_range(5..10),
        ];
        let parser = Parser { fields, width: 10 };
        let map = parser.parse("1234567890");

        assert_eq!(map.len(), 2);
        assert!(map.contains_key("test-1"));
        assert_eq!(map.get("test-1"), Some(&String::from("12345")));
        assert!(map.contains_key("test-2"));
        assert_eq!(map.get("test-2"), Some(&String::from("67890")));
    }

    #[test]
    fn check_parsing_field_with_spacer() {
        let fields = vec![
            Field::default().with_range(0..5),
            Field::default().with_name("test").with_range(5..10),
        ];
        let parser = Parser { fields, width: 10 };
        let map = parser.parse("1234567890");

        assert_eq!(map.len(), 1);
        assert!(map.contains_key("test"));
        assert_eq!(map.get("test"), Some(&String::from("67890")));
    }

    #[test]
    #[should_panic(expected = "Insufficient buffer allocated")]
    fn check_parsing_small_buffer() {
        let fields = vec![
            Field::default().with_range(0..5),
            Field::default().with_name("test").with_range(5..10),
        ];
        let parser = Parser { fields, width: 10 };
        parser.parse("1234567");
    }

    #[test]
    fn check_field_parsing() {
        let field = Field::default().with_name("test").with_range(0..5);
        let mut map = HashMap::new();
        field.parse(&mut map, &mut "1234567890".chars());

        assert!(map.contains_key("test"));
        assert_eq!(map.get("test"), Some(&String::from("12345")));
    }

    #[test]
    fn check_field_default() {
        let field = Field::default();

        assert_eq!(field.name, None);
        assert_eq!(field.width, 0);
        assert_eq!(field.align, Align::Left);
        assert_eq!(field.padding, ' ');
    }

    #[test]
    fn check_field_with_name() {
        let field = Field::default().with_name("foo");

        assert_eq!(field.name(), Some("foo"));
    }

    #[test]
    fn check_field_with_width() {
        let field = Field::default().with_width(20);

        assert_eq!(field.width(), 20);
    }

    #[test]
    fn check_field_with_range() {
        let field = Field::default().with_range(5..20);

        assert_eq!(field.width(), 15);
    }

    #[test]
    fn check_field_with_align() {
        let field = Field::default().with_align(Align::Right);

        assert_eq!(field.align(), Align::Right);
    }

    #[test]
    fn check_field_with_align_from_string() {
        let field = Field::default().with_align("right");

        assert_eq!(field.align(), Align::Right);
    }

    #[test]
    fn check_field_with_align_error() {
        let field = Field::default().with_align("banana");

        assert_eq!(field.align(), Align::Left);
    }

    #[test]
    fn check_field_with_padding() {
        let field = Field::default().with_padding('X');

        assert_eq!(field.padding(), 'X');
    }

    #[test]
    fn check_field_without_name() {
        let field = Field::default().with_name("foo").without_name();

        assert_eq!(field.name(), None);
    }

    #[test]
    fn check_field_chained() {
        let field = Field::default()
            .with_name("foo")
            .with_align("right")
            .with_range(10..30)
            .with_padding('X');

        assert_eq!(field.name(), Some("foo"));
        assert_eq!(field.width(), 20);
        assert_eq!(field.align(), Align::Right);
        assert_eq!(field.padding(), 'X');
    }

    #[test]
    fn check_field_new() {
        let field = Field::new(Some("foo"), 20, Align::Right, 'X');

        assert_eq!(field.name(), Some("foo"));
        assert_eq!(field.width(), 20);
        assert_eq!(field.align(), Align::Right);
        assert_eq!(field.padding(), 'X');
    }
}
