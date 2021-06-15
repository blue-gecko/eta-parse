use crate::{
    error::{Error, ParseError},
    utilities::string::{fixed_width, strip_padding, Align},
};
use std::{
    collections::HashMap,
    convert::{From, Into, TryInto},
    fmt::Debug,
    ops::Range,
    result::Result,
    str::Chars,
};

mod builder;
mod read;
mod write;

pub type Record = HashMap<String, String>;
pub type ResultRecord = Result<Record, Error>;

#[derive(Debug)]
pub struct Parser<'a> {
    fields: Vec<Field<'a>>,
    width: usize,
}

#[allow(dead_code)]
impl<'a> Parser<'a> {
    fn parse<T: Into<String>>(&self, s: T) -> ResultRecord {
        let s: String = s.into();
        let mut iter = s.chars();
        let c = &mut iter.by_ref();
        match c.size_hint() {
            (_, Some(max)) if max < self.width => {
                return Err(Error::from(ParseError::ImsufficentBuffer(
                    self.width,
                    Some(max),
                )))
            }
            (_, None) => return Err(Error::from(ParseError::ImsufficentBuffer(self.width, None))),
            _ => (),
        }

        let mut map = HashMap::new();
        for field in &self.fields {
            field.parse(&mut map, c);
        }
        Ok(map)
    }

    fn format(&self, data: Record) -> String {
        self.fields
            .iter()
            .fold(String::with_capacity(self.width), |mut s, f| {
                s.push_str(&*f.format(&data));
                s
            })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Field<'a> {
    name: Option<&'a str>,
    width: usize,
    align: Align,
    padding: char,
}

#[allow(dead_code)]
impl<'a> Field<'a> {
    fn new(name: Option<&'a str>, width: usize, align: Align, padding: char) -> Self {
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

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn with_range(mut self, range: Range<usize>) -> Self {
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

    pub fn width(&self) -> usize {
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
            map.entry(name.to_string()).or_insert_with(|| {
                strip_padding(
                    &chars.take(width).collect::<String>(),
                    self.align(),
                    self.padding(),
                )
                .to_string()
            });
        } else {
            chars.take(width).for_each(|_| {});
        }
    }

    fn format(&self, data: &Record) -> String {
        let mut s = String::with_capacity(self.width() as usize);
        if let Some(name) = self.name() {
            if let Some(data) = data.get(name) {
                s.push_str(data);
            }
        }
        fixed_width(&*s, self.width(), self.align(), self.padding()).to_string()
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
    fn check_parser() {
        let parser = Parser {
            fields: Vec::new(),
            width: 0,
        };

        assert_eq!(parser.fields.len(), 0);
    }

    #[test]
    fn check_format() {
        let fields = vec![Field::default().with_name("test").with_range(0..10)];
        let parser = Parser { fields, width: 10 };
        let data: HashMap<String, String> = [(String::from("test"), String::from("ABCD"))]
            .iter()
            .cloned()
            .collect();

        assert_eq!(parser.format(data), "ABCD      ");
    }

    #[test]
    fn check_format_two_fields() {
        let fields = vec![
            Field::default().with_name("test-1").with_range(0..5),
            Field::default()
                .with_name("test-2")
                .with_range(5..10)
                .with_align(Align::Right)
                .with_padding('0'),
        ];
        let parser = Parser { fields, width: 10 };
        let data: HashMap<String, String> = [
            (String::from("test-1"), String::from("ABCD")),
            (String::from("test-2"), String::from("1234")),
        ]
        .iter()
        .cloned()
        .collect();

        assert_eq!(parser.format(data), "ABCD 01234");
    }

    #[test]
    fn check_format_missing_data() {
        let fields = vec![
            Field::default().with_name("test-1").with_range(0..5),
            Field::default()
                .with_name("test-2")
                .with_range(5..10)
                .with_align(Align::Right)
                .with_padding('0'),
        ];
        let parser = Parser { fields, width: 10 };
        let data: HashMap<String, String> = [(String::from("test-1"), String::from("ABCD"))]
            .iter()
            .cloned()
            .collect();

        assert_eq!(parser.format(data), "ABCD 00000");
    }

    #[test]
    fn check_parsing() {
        let fields = vec![Field::default().with_name("test").with_range(0..10)];
        let parser = Parser { fields, width: 10 };

        if let Ok(map) = parser.parse("1234567890") {
            assert!(map.contains_key("test"));
            assert_eq!(map.get("test"), Some(&String::from("1234567890")));
        }
    }

    #[test]
    fn check_parsing_two_fields() {
        let fields = vec![
            Field::default().with_name("test-1").with_range(0..5),
            Field::default().with_name("test-2").with_range(5..10),
        ];
        let parser = Parser { fields, width: 10 };

        if let Ok(map) = parser.parse("1234567890") {
            assert_eq!(map.len(), 2);
            assert!(map.contains_key("test-1"));
            assert_eq!(map.get("test-1"), Some(&String::from("12345")));
            assert!(map.contains_key("test-2"));
            assert_eq!(map.get("test-2"), Some(&String::from("67890")));
        }
    }

    #[test]
    fn check_parsing_field_with_spacer() {
        let fields = vec![
            Field::default().with_range(0..5),
            Field::default().with_name("test").with_range(5..10),
        ];
        let parser = Parser { fields, width: 10 };

        if let Ok(map) = parser.parse("1234567890") {
            assert_eq!(map.len(), 1);
            assert!(map.contains_key("test"));
            assert_eq!(map.get("test"), Some(&String::from("67890")));
        }
    }

    #[test]
    fn check_parsing_small_buffer() {
        let fields = vec![
            Field::default().with_range(0..5),
            Field::default().with_name("test").with_range(5..10),
        ];
        let parser = Parser { fields, width: 10 };
        if let Err(e) = parser.parse("1234567") {
            assert!(matches!(e, Error::ParserError(_)));
            assert_eq!(
                e.to_string(),
                "Insufficient buffer size, required 10 only 7 available"
            );
        }
    }

    #[test]
    fn check_field_parsing_with_padding_left() {
        let field = Field::default().with_name("test").with_range(0..5);
        let mut map = HashMap::new();
        field.parse(&mut map, &mut "A    BCDEF".chars());

        assert!(map.contains_key("test"));
        assert_eq!(map.get("test"), Some(&String::from("A")));
    }

    #[test]
    fn check_field_parsing_with_padding_right() {
        let field = Field::default()
            .with_name("test")
            .with_range(0..5)
            .with_align(Align::Right)
            .with_padding('X');
        let mut map = HashMap::new();
        field.parse(&mut map, &mut "XXXX123456".chars());

        assert!(map.contains_key("test"));
        assert_eq!(map.get("test"), Some(&String::from("1")));
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
    fn check_field_format() {
        let field = Field::default().with_name("test-1").with_range(0..5);
        let data: HashMap<String, String> = [
            (String::from("test-1"), String::from("ABCD")),
            (String::from("test-2"), String::from("1234")),
        ]
        .iter()
        .cloned()
        .collect();

        assert_eq!(field.format(&data), "ABCD ");
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
