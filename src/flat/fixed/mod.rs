use std::collections::HashMap;
use std::convert::{From, Into, TryFrom, TryInto};
use std::fmt::Debug;
use std::ops::Range;

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
}

// #[allow(dead_code)]
impl<'a> Parser<'a> {
    fn parse<T: Into<String>>(&self, s: T) -> HashMap<String, String> {
        let mut map = HashMap::new();
        let s = s.into();
        for field in &self.fields {
            field.parse(&mut map, &s);
        }
        map
    }
}

#[allow(dead_code)]
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

#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    index: Option<u32>,
    name: Option<&'a str>,
    position: Option<u32>,
    width: Option<u32>,
    align: Align,
    padding: char,
}

#[allow(dead_code)]
impl<'a> Field<'a> {
    fn new(
        name: Option<&'a str>,
        position: Option<u32>,
        width: Option<u32>,
        align: Align,
        padding: char,
    ) -> Self {
        Field::_raw(None, name, position, width, align, padding)
    }

    fn _raw(
        index: Option<u32>,
        name: Option<&'a str>,
        position: Option<u32>,
        width: Option<u32>,
        align: Align,
        padding: char,
    ) -> Self {
        Field {
            index,
            name,
            position,
            width,
            align,
            padding,
        }
    }

    pub fn with_index(mut self, index: u32) -> Self {
        self.index = Some(index);
        self
    }

    pub fn without_index(mut self) -> Self {
        self.index = None;
        self
    }

    pub fn with_name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn without_name(mut self) -> Self {
        self.name = None;
        self
    }

    pub fn with_position(mut self, position: u32) -> Self {
        self.position = Some(position);
        self
    }

    pub fn without_position(mut self) -> Self {
        self.position = None;
        self
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }
    pub fn without_width(mut self) -> Self {
        self.width = None;
        self
    }

    pub fn with_range(mut self, range: Range<u32>) -> Self {
        self.position = Some(range.start);
        self.width = Some(range.end - range.start);
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

    pub fn index(&self) -> u32 {
        self.index.expect("Index should be set before use")
    }

    pub fn name(&self) -> Option<&str> {
        self.name
    }

    pub fn position(&self) -> Option<u32> {
        self.position
    }

    pub fn width(&self) -> Option<u32> {
        self.width
    }

    pub fn align(&self) -> Align {
        self.align
    }

    pub fn padding(&self) -> char {
        self.padding
    }

    pub fn range(&self) -> Range<usize> {
        if let (Some(position), Some(width)) = (self.position(), self.width()) {
            Range {
                start: position as usize,
                end: (position + width) as usize,
            }
        } else {
            Range { start: 0, end: 0 }
        }
    }

    fn parse<T: Into<String>>(&self, map: &mut HashMap<String, String>, s: T) {
        if let Some(name) = self.name {
            if let Some(extract) = s.into().get(self.range()) {
                map.entry(name.to_string())
                    .or_insert_with(|| extract.to_string());
            }
        }
    }
}

impl<'a> Default for Field<'a> {
    fn default() -> Self {
        Self {
            index: None,
            name: None,
            position: None,
            width: None,
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
        let parser = Parser { fields: Vec::new() };

        assert_eq!(parser.fields.len(), 0);
    }

    #[test]
    fn check_parsing() {
        let fields = vec![Field::default()
            .with_index(0)
            .with_name("test")
            .with_range(0..10)];
        let parser = Parser { fields };
        let map = parser.parse("1234567890");

        assert!(map.contains_key("test"));
        assert_eq!(map.get("test"), Some(&String::from("1234567890")));
    }

    #[test]
    fn check_field_parsing() {
        let field = Field::default().with_name("test").with_range(0..5);
        let mut map = HashMap::new();
        field.parse(&mut map, "1234567890");

        assert!(map.contains_key("test"));
        assert_eq!(map.get("test"), Some(&String::from("12345")));
    }

    #[test]
    fn check_field_default() {
        let field = Field::default();

        assert_eq!(field.index, None);
        assert_eq!(field.name, None);
        assert_eq!(field.position, None);
        assert_eq!(field.width, None);
        assert_eq!(field.align, Align::Left);
        assert_eq!(field.padding, ' ');
    }

    #[test]
    fn check_field_with_index() {
        let field = Field::default().with_index(1);

        assert_eq!(field.index(), 1);
    }

    #[test]
    fn check_field_with_name() {
        let field = Field::default().with_name("foo");

        assert_eq!(field.name(), Some("foo"));
    }

    #[test]
    fn check_field_with_position() {
        let field = Field::default().with_position(10);

        assert_eq!(field.position(), Some(10));
    }

    #[test]
    fn check_field_with_width() {
        let field = Field::default().with_width(20);

        assert_eq!(field.width(), Some(20));
    }

    #[test]
    fn check_field_with_range() {
        let field = Field::default().with_range(5..20);

        assert_eq!(field.position(), Some(5));
        assert_eq!(field.width(), Some(15));
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
    fn check_field_without_index() {
        let field = Field::default().with_index(1).without_index();

        assert_eq!(field.index, None);
    }

    #[test]
    fn check_field_without_name() {
        let field = Field::default().with_name("foo").without_name();

        assert_eq!(field.name(), None);
    }

    #[test]
    fn check_field_without_position() {
        let field = Field::default().with_position(10).without_position();

        assert_eq!(field.position(), None);
    }

    #[test]
    fn check_field_without_width() {
        let field = Field::default().with_width(20).without_width();

        assert_eq!(field.width(), None);
    }

    #[test]
    fn check_field_chained() {
        let field = Field::default()
            .with_index(2)
            .with_name("foo")
            .with_align("right")
            .with_range(10..30)
            .with_padding('X');

        assert_eq!(field.index(), 2);
        assert_eq!(field.name(), Some("foo"));
        assert_eq!(field.position(), Some(10));
        assert_eq!(field.width(), Some(20));
        assert_eq!(field.align(), Align::Right);
        assert_eq!(field.padding(), 'X');
    }

    #[test]
    fn check_field_raw() {
        let field = Field::_raw(Some(2), Some("foo"), Some(10), Some(20), Align::Right, 'X');
        assert_eq!(field.index(), 2);
        assert_eq!(field.name(), Some("foo"));
        assert_eq!(field.position(), Some(10));
        assert_eq!(field.width(), Some(20));
        assert_eq!(field.align(), Align::Right);
        assert_eq!(field.padding(), 'X');
    }

    #[test]
    fn check_field_new() {
        let field = Field::new(Some("foo"), Some(10), Some(20), Align::Right, 'X');

        assert_eq!(field.index, None);
        assert_eq!(field.name(), Some("foo"));
        assert_eq!(field.position(), Some(10));
        assert_eq!(field.width(), Some(20));
        assert_eq!(field.align(), Align::Right);
        assert_eq!(field.padding(), 'X');
    }
}
