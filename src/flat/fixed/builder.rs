use crate::{
    builder::{Buildable, Builder},
    flat::fixed::{Align, Field, Parser},
};
use std::{convert::TryInto, ops::Range};

impl<'a> Buildable for Parser<'a> {
    type Builder = ParserBuilder<'a>;

    fn builder() -> Self::Builder {
        ParserBuilder::new()
    }
}

#[derive(Debug)]
pub struct ParserBuilder<'a> {
    fields: Vec<Field<'a>>,
    align: Align,
    padding: char,
}

#[allow(dead_code)]
impl<'a> ParserBuilder<'a> {
    pub fn new() -> Self {
        ParserBuilder {
            fields: Vec::new(),
            align: Align::Left,
            padding: ' ',
        }
    }

    fn append(mut self, field: Field<'a>) -> Self {
        self.fields.push(field);
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

    pub fn spacer(self, range: Range<usize>) -> Self {
        let align = self.align;
        let padding = self.padding;
        self.append(Field::new(None, range.end - range.start, align, padding))
    }
}

impl<'a> Builder for ParserBuilder<'a> {
    type Target = Parser<'a>;

    fn build(&mut self) -> Self::Target {
        let mut width = 0;
        Parser {
            fields: self
                .fields
                .iter()
                .copied()
                .inspect(|f| width += f.width)
                .collect(),
            width: width as usize,
        }
    }
}

pub struct FieldBuilder<'a> {
    parser: ParserBuilder<'a>,
    name: Option<&'a str>,
    width: Option<usize>,
    align: Align,
    padding: char,
}

#[allow(dead_code)]
impl<'a> FieldBuilder<'a> {
    fn new(parser: ParserBuilder<'a>, name: Option<&'a str>, align: Align, padding: char) -> Self {
        FieldBuilder {
            parser,
            name,
            width: None,
            align,
            padding,
        }
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    pub fn range(mut self, range: Range<usize>) -> Self {
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
            self.width.expect("Width must be specified"),
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
            Field::new(Some("first"), 20, Align::Left, ' ')
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
            Field::new(Some("first"), 20, Align::Right, '-')
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
            Field::new(Some("first"), 20, Align::Right, ' ')
        );
    }

    #[test]
    fn check_field_width() {
        let parser = Parser::builder().field("first").width(10).append().build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::new(Some("first"), 10, Align::Left, ' ')
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
            Field::new(Some("first"), 15, Align::Left, ' ')
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
            Field::new(Some("first"), 20, Align::Right, ' ')
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
            Field::new(Some("first"), 20, Align::Left, 'X')
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
            Field::new(Some("first"), 20, Align::Left, ' ')
        );
        assert_eq!(
            parser.fields[1],
            Field::new(Some("second"), 30, Align::Right, '0')
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
            Field::new(Some("second"), 30, Align::Right, 'X')
        );
        assert_eq!(
            parser.fields[1],
            Field::new(Some("first"), 20, Align::Left, ' ')
        );
    }

    #[test]
    fn check_field_two_stage() {
        let mut builder = Parser::builder().field("first").width(20).append();

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
            Field::new(Some("first"), 20, Align::Left, ' ')
        );
        assert_eq!(
            parser.fields[1],
            Field::new(Some("second"), 30, Align::Right, '0')
        );
    }

    #[test]
    #[should_panic(expected = "Width must be specified")]
    fn check_field_one_missing_width() {
        Parser::builder().field("first").append().build();
    }

    #[test]
    fn check_spacer() {
        let parser = Parser::builder().spacer(5..15).build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(parser.fields[0], Field::new(None, 10, Align::Left, ' '));
    }
}
