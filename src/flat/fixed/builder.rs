use super::*;
use std::collections::VecDeque;
use std::ops::Range;

trait Builder {
    type Builder;

    fn builder() -> Self::Builder;
}

#[allow(dead_code)]
impl<'a> Builder for Parser<'a> {
    type Builder = ParserBuilder<'a>;

    fn builder() -> Self::Builder {
        builder::ParserBuilder::new()
    }
}

#[derive(Debug)]
pub struct ParserBuilder<'a> {
    fields: VecDeque<Field<'a>>,
    justify: Justify,
    padding: char,
}

#[allow(dead_code)]
impl<'a> ParserBuilder<'a> {
    pub fn new() -> Self {
        ParserBuilder {
            fields: VecDeque::new(),
            justify: Justify::Left,
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

    pub fn build(&mut self) -> Parser<'a> {
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

                    if let Some(mut previous) = fields.last_mut() {
                        if previous.width.is_none() {
                            previous.width = Some(
                                position
                                    - previous
                                        .position
                                        .expect("Either position or width must be specified"),
                            );
                        }
                    }
                }
            } else {
                if current.width.is_none() {
                    panic!("Either position or width must be specified");
                }
                current.position = Some(position);
            }
            if let Some(w) = current.width {
                position += w;
            }

            fields.push(current);
        }
        Parser { fields }
    }

    pub fn default_justify<T: TryInto<Justify> + Debug>(mut self, justify: T) -> Self {
        match justify.try_into() {
            Ok(justify) => self.justify = justify,
            Err(_) => eprintln!("Unable to parse argument as Justify"),
        }
        self
    }

    pub fn default_padding<T: Into<char>>(mut self, padding: T) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn field<T: Into<&'a str>>(self, name: T) -> FieldBuilder<'a> {
        let justify = self.justify;
        let padding = self.padding;
        FieldBuilder::new(self, Some(name.into()), justify, padding)
    }

    pub fn spacer(self, range: Range<u32>) -> FieldBuilder<'a> {
        let justify = self.justify;
        let padding = self.padding;
        FieldBuilder::new(self, None, justify, padding).range(range)
    }
}

#[allow(dead_code)]
pub struct FieldBuilder<'a> {
    parser: ParserBuilder<'a>,
    name: Option<&'a str>,
    position: Option<u32>,
    width: Option<u32>,
    justify: Justify,
    padding: char,
}

#[allow(dead_code)]
impl<'a> FieldBuilder<'a> {
    fn new(
        parser: ParserBuilder<'a>,
        name: Option<&'a str>,
        justify: Justify,
        padding: char,
    ) -> Self {
        FieldBuilder {
            parser,
            name,
            position: None,
            width: None,
            justify,
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

    pub fn justify<T: TryInto<Justify>>(mut self, justify: T) -> Self {
        match justify.try_into() {
            Ok(justify) => self.justify = justify,
            Err(_) => eprintln!("Unable to parse argument as Justify"),
        }
        self
    }

    pub fn padding<T: Into<char>>(mut self, padding: T) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn append(self) -> ParserBuilder<'a> {
        let field = self.build();
        self.parser.append(field)
    }

    pub fn insert(self, index: usize) -> ParserBuilder<'a> {
        let field = self.build();
        self.parser.insert(index, field)
    }

    fn build(&self) -> Field<'a> {
        Field::new(
            self.name,
            self.position,
            self.width,
            self.justify,
            self.padding,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_field_one() {
        let parser = Parser::builder().field("first").width(20).append().build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_raw(
                Some(0),
                Some("first"),
                Some(0),
                Some(20),
                Justify::Left,
                ' '
            )
        );
    }

    #[test]
    fn check_field_one_overide() {
        let parser = Parser::builder()
            .default_justify(Justify::Right)
            .default_padding('-')
            .field("first")
            .width(20)
            .append()
            .build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_raw(
                Some(0),
                Some("first"),
                Some(0),
                Some(20),
                Justify::Right,
                '-'
            )
        );
    }

    #[test]
    fn check_field_position() {
        let parser = Parser::builder()
            .field("first")
            .position(20)
            .append()
            .build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_raw(Some(0), Some("first"), Some(20), None, Justify::Left, ' ')
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
            Field::_raw(
                Some(0),
                Some("first"),
                Some(5),
                Some(15),
                Justify::Left,
                ' '
            )
        );
    }

    #[test]
    fn check_field_justify() {
        let parser = Parser::builder()
            .field("first")
            .width(20)
            .justify("right")
            .append()
            .build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_raw(
                Some(0),
                Some("first"),
                Some(0),
                Some(20),
                Justify::Right,
                ' '
            )
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
            Field::_raw(
                Some(0),
                Some("first"),
                Some(0),
                Some(20),
                Justify::Left,
                'X'
            )
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
            .justify(Justify::Right)
            .padding('0')
            .append()
            .build();

        assert_eq!(parser.fields.len(), 2);
        assert_eq!(
            parser.fields[0],
            Field::_raw(
                Some(0),
                Some("first"),
                Some(0),
                Some(20),
                Justify::Left,
                ' '
            )
        );
        assert_eq!(
            parser.fields[1],
            Field::_raw(
                Some(1),
                Some("second"),
                Some(20),
                Some(30),
                Justify::Right,
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
            .justify(Justify::Right)
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
                Justify::Right,
                'X'
            )
        );
        assert_eq!(
            parser.fields[1],
            Field::_raw(
                Some(1),
                Some("first"),
                Some(50),
                Some(20),
                Justify::Left,
                ' '
            )
        );
    }

    #[test]
    fn check_field_two_stage() {
        let mut builder = Parser::builder().field("first").width(20).append();

        builder = builder
            .field("second")
            .range(20..50)
            .justify(Justify::Right)
            .padding('0')
            .append();

        let parser = builder.build();

        assert_eq!(parser.fields.len(), 2);
        assert_eq!(
            parser.fields[0],
            Field::_raw(
                Some(0),
                Some("first"),
                Some(0),
                Some(20),
                Justify::Left,
                ' '
            )
        );
        assert_eq!(
            parser.fields[1],
            Field::_raw(
                Some(1),
                Some("second"),
                Some(20),
                Some(30),
                Justify::Right,
                '0'
            )
        );
    }

    #[test]
    fn check_spacer() {
        let parser = Parser::builder().spacer(5..15).append().build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_raw(Some(0), None, Some(5), Some(10), Justify::Left, ' ')
        );
    }
}
