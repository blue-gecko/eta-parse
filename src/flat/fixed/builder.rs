use super::*;
use std::collections::VecDeque;
use std::ops::Range;

use crate::builder::{Buildable, Builder};

#[allow(dead_code)]
impl<'a> Buildable for Parser<'a> {
    type Builder = ParserBuilder<'a>;

    fn builder() -> Self::Builder {
        ParserBuilder::new()
    }
}

#[derive(Debug)]
pub struct ParserBuilder<'a> {
    fields: VecDeque<Field<'a>>,
    align: Align,
    padding: char,
}

#[allow(dead_code)]
impl<'a> ParserBuilder<'a> {
    pub fn new() -> Self {
        ParserBuilder {
            fields: VecDeque::new(),
            align: Align::Left,
            padding: ' ',
        }
    }

    fn append(mut self, field: Field<'a>) -> Self {
        self.fields.push_back(field);
        self
    }

    fn insert(mut self, index: usize, field: Field<'a>) -> Self {
        self.fields.insert(index, field);
        self
    }

    pub fn default_align<T: TryInto<Align>>(mut self, align: T) -> Self {
        match align.try_into() {
            Ok(align) => self.align = align,
            Err(_) => eprintln!("Unable to parse argument as Align"),
        }
        self
    }

    pub fn default_padding<T: Into<char>>(mut self, padding: T) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn field(self, name: &'a str) -> FieldBuilder<'a> {
        let align = self.align;
        let padding = self.padding;
        FieldBuilder::new(self, Some(name), align, padding)
    }

    pub fn spacer(self, range: Range<u32>) -> FieldBuilder<'a> {
        let align = self.align;
        let padding = self.padding;
        FieldBuilder::new(self, None, align, padding).range(range)
    }
}

impl<'a> Builder for ParserBuilder<'a> {
    type Target = Parser<'a>;

    fn build(&mut self) -> Self::Target {
        let mut fields: Vec<Field> = Vec::new();
        let mut position: u32 = 0;
        let mut index = 0;
        while let Some(mut current) = self.fields.pop_front() {
            current.index = Some(index);
            index += 1;
            if let Some(p) = current.position {
                if p < position {
                    panic!("Position before current marker");
                } else {
                    position = p;
                }
            } else {
                current.position = Some(position);
            }

            if let Some(w) = current.width {
                position += w;
            } else {
                panic!("Width must be defined");
            }

            fields.push(current);
        }
        Parser { fields }
    }
}

#[allow(dead_code)]
pub struct FieldBuilder<'a> {
    parser: ParserBuilder<'a>,
    name: Option<&'a str>,
    position: Option<u32>,
    width: Option<u32>,
    align: Align,
    padding: char,
}

#[allow(dead_code)]
impl<'a> FieldBuilder<'a> {
    fn new(parser: ParserBuilder<'a>, name: Option<&'a str>, align: Align, padding: char) -> Self {
        FieldBuilder {
            parser,
            name,
            position: None,
            width: None,
            align,
            padding,
        }
    }

    pub fn position(mut self, position: u32) -> Self {
        self.position = Some(position);
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn range(mut self, range: Range<u32>) -> Self {
        self.position = Some(range.start);
        self.width = Some(range.end - range.start);
        self
    }

    pub fn align<T: TryInto<Align>>(mut self, align: T) -> Self {
        match align.try_into() {
            Ok(align) => self.align = align,
            Err(_) => eprintln!("Unable to parse argument as Align"),
        }
        self
    }

    pub fn padding<T: Into<char>>(mut self, padding: T) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn append(mut self) -> ParserBuilder<'a> {
        let field = self.build();
        self.parser.append(field)
    }

    pub fn insert(mut self, index: usize) -> ParserBuilder<'a> {
        let field = self.build();
        self.parser.insert(index, field)
    }
}

impl<'a> Builder for FieldBuilder<'a> {
    type Target = Field<'a>;

    fn build(&mut self) -> Self::Target {
        Field::new(
            self.name,
            self.position,
            self.width,
            self.align,
            self.padding,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_builder() {
        let builder = Parser::builder();

        assert_eq!(builder.align, Align::Left);
        assert_eq!(builder.padding, ' ');
    }

    #[test]
    fn check_builder_padding() {
        let builder = Parser::builder().default_padding('X');

        assert_eq!(builder.padding, 'X');
    }

    #[test]
    fn check_builder_align() {
        let builder = Parser::builder().default_align(Align::Right);

        assert_eq!(builder.align, Align::Right);
    }

    #[test]
    fn check_builder_align_from_string() {
        let builder = Parser::builder().default_align("RIGHT");

        assert_eq!(builder.align, Align::Right);
    }

    #[test]
    fn check_builder_align_fail() {
        let builder = Parser::builder().default_align("banana");

        assert_eq!(builder.align, Align::Left);
    }

    #[test]
    fn check_field_one() {
        let parser = Parser::builder().field("first").width(20).append().build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_raw(Some(0), Some("first"), Some(0), Some(20), Align::Left, ' ')
        );
    }

    #[test]
    fn check_field_one_overide() {
        let parser = Parser::builder()
            .default_align(Align::Right)
            .default_padding('-')
            .field("first")
            .width(20)
            .append()
            .build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_raw(Some(0), Some("first"), Some(0), Some(20), Align::Right, '-')
        );
    }

    #[test]
    fn check_field_one_align_fail() {
        let parser = Parser::builder()
            .default_align(Align::Right)
            .field("first")
            .width(20)
            .align("banana")
            .append()
            .build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_raw(Some(0), Some("first"), Some(0), Some(20), Align::Right, ' ')
        );
    }

    #[test]
    fn check_field_position() {
        let parser = Parser::builder()
            .field("first")
            .position(20)
            .width(10)
            .append()
            .build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_raw(Some(0), Some("first"), Some(20), Some(10), Align::Left, ' ')
        );
    }

    #[test]
    fn check_field_range() {
        let parser = Parser::builder()
            .field("first")
            .range(5..20)
            .append()
            .build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_raw(Some(0), Some("first"), Some(5), Some(15), Align::Left, ' ')
        );
    }

    #[test]
    fn check_field_align() {
        let parser = Parser::builder()
            .field("first")
            .width(20)
            .align("right")
            .append()
            .build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_raw(Some(0), Some("first"), Some(0), Some(20), Align::Right, ' ')
        );
    }

    #[test]
    fn check_field_padding() {
        let parser = Parser::builder()
            .field("first")
            .width(20)
            .padding('X')
            .append()
            .build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_raw(Some(0), Some("first"), Some(0), Some(20), Align::Left, 'X')
        );
    }

    #[test]
    fn check_field_two() {
        let parser = Parser::builder()
            .field("first")
            .width(20)
            .append()
            .field("second")
            .range(20..50)
            .align(Align::Right)
            .padding('0')
            .append()
            .build();

        assert_eq!(parser.fields.len(), 2);
        assert_eq!(
            parser.fields[0],
            Field::_raw(Some(0), Some("first"), Some(0), Some(20), Align::Left, ' ')
        );
        assert_eq!(
            parser.fields[1],
            Field::_raw(
                Some(1),
                Some("second"),
                Some(20),
                Some(30),
                Align::Right,
                '0'
            )
        );
    }

    #[test]
    fn check_field_insert() {
        let parser = Parser::builder()
            .field("first")
            .width(20)
            .append()
            .field("second")
            .range(20..50)
            .align(Align::Right)
            .padding('X')
            .insert(0)
            .build();

        assert_eq!(parser.fields.len(), 2);
        assert_eq!(
            parser.fields[0],
            Field::_raw(
                Some(0),
                Some("second"),
                Some(20),
                Some(30),
                Align::Right,
                'X'
            )
        );
        assert_eq!(
            parser.fields[1],
            Field::_raw(Some(1), Some("first"), Some(50), Some(20), Align::Left, ' ')
        );
    }

    #[test]
    fn check_field_two_stage() {
        let mut builder = Parser::builder()
            .field("first")
            .position(0)
            .width(20)
            .append();

        builder = builder
            .field("second")
            .range(20..50)
            .align(Align::Right)
            .padding('0')
            .append();

        let parser = builder.build();

        assert_eq!(parser.fields.len(), 2);
        assert_eq!(
            parser.fields[0],
            Field::_raw(Some(0), Some("first"), Some(0), Some(20), Align::Left, ' ')
        );
        assert_eq!(
            parser.fields[1],
            Field::_raw(
                Some(1),
                Some("second"),
                Some(20),
                Some(30),
                Align::Right,
                '0'
            )
        );
    }

    #[test]
    #[should_panic(expected = "Width must be defined")]
    fn check_field_one_missing_width() {
        Parser::builder().field("first").append().build();
    }

    #[test]
    #[should_panic(expected = "Position before current marker")]
    fn check_field_two_position_error() {
        Parser::builder()
            .field("first")
            .range(0..10)
            .append()
            .field("second")
            .position(5)
            .width(10)
            .append()
            .build();
    }

    #[test]
    fn check_spacer() {
        let parser = Parser::builder().spacer(5..15).append().build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_raw(Some(0), None, Some(5), Some(10), Align::Left, ' ')
        );
    }
}
