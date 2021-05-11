use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::ops::Range;

mod builder;

#[derive(Debug)]
pub struct Parser<'a> {
    fields: Vec<Field<'a>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Justify {
    Left,
    Right,
}

impl TryFrom<&str> for Justify {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().trim() {
            "left" => Ok(Justify::Left),
            "right" => Ok(Justify::Right),
            _ => Err(String::from("Unknown justify argument")),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    index: Option<u32>,
    name: Option<&'a str>,
    position: Option<u32>,
    width: Option<u32>,
    justify: Justify,
    padding: char,
}

#[allow(dead_code)]
impl<'a> Field<'a> {
    fn new(
        name: Option<&'a str>,
        position: Option<u32>,
        width: Option<u32>,
        justify: Justify,
        padding: char,
    ) -> Self {
        Field::_raw(None, name, position, width, justify, padding)
    }

    fn _raw(
        index: Option<u32>,
        name: Option<&'a str>,
        position: Option<u32>,
        width: Option<u32>,
        justify: Justify,
        padding: char,
    ) -> Self {
        Field {
            index,
            name,
            position,
            width,
            justify,
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

    pub fn with_name<T: Into<&'a str>>(mut self, name: T) -> Self {
        self.name = Some(name.into());
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

    pub fn with_justify<T: TryInto<Justify>>(mut self, justify: T) -> Self {
        match justify.try_into() {
            Ok(justify) => self.justify = justify,
            Err(_) => eprintln!("Unable to parse argument as Justify"),
        }
        self
    }

    pub fn with_padding<T: Into<char>>(mut self, padding: T) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn index(&self) -> u32 {
        self.index.expect("Index should have been set before use")
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

    pub fn justify(&self) -> Justify {
        self.justify
    }

    pub fn padding(&self) -> char {
        self.padding
    }
}

impl<'a> Default for Field<'a> {
    fn default() -> Self {
        Self {
            index: None,
            name: None,
            position: None,
            width: None,
            justify: Justify::Left,
            padding: ' ',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_justify_into() {
        assert_eq!(Justify::try_from("LEFT"), Ok(Justify::Left));
        assert_eq!(Justify::try_from("Right"), Ok(Justify::Right));
        assert!(matches!(Justify::try_from("Banana"), Err(_)));
    }

    #[test]
    fn check_field_default() {
        let field = Field::default();

        assert_eq!(field.index, None);
        assert_eq!(field.name, None);
        assert_eq!(field.position, None);
        assert_eq!(field.width, None);
        assert_eq!(field.justify, Justify::Left);
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
    fn check_field_with_justify() {
        let field = Field::default().with_justify(Justify::Right);

        assert_eq!(field.justify(), Justify::Right);
    }

    #[test]
    fn check_field_with_justify_from_string() {
        let field = Field::default().with_justify("right");

        assert_eq!(field.justify(), Justify::Right);
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
            .with_justify("right")
            .with_range(10..30)
            .with_padding('X');

        assert_eq!(field.index(), 2);
        assert_eq!(field.name(), Some("foo"));
        assert_eq!(field.position(), Some(10));
        assert_eq!(field.width(), Some(20));
        assert_eq!(field.justify(), Justify::Right);
        assert_eq!(field.padding(), 'X');
    }

    #[test]
    fn check_field_raw() {
        let field = Field::_raw(
            Some(2),
            Some("foo"),
            Some(10),
            Some(20),
            Justify::Right,
            'X',
        );
        assert_eq!(field.index(), 2);
        assert_eq!(field.name(), Some("foo"));
        assert_eq!(field.position(), Some(10));
        assert_eq!(field.width(), Some(20));
        assert_eq!(field.justify(), Justify::Right);
        assert_eq!(field.padding(), 'X');
    }

    #[test]
    fn check_field_new() {
        let field = Field::new(Some("foo"), Some(10), Some(20), Justify::Right, 'X');

        assert_eq!(field.index, None);
        assert_eq!(field.name(), Some("foo"));
        assert_eq!(field.position(), Some(10));
        assert_eq!(field.width(), Some(20));
        assert_eq!(field.justify(), Justify::Right);
        assert_eq!(field.padding(), 'X');
    }
}
