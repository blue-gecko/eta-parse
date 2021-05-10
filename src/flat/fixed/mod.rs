#[derive(Debug)]
pub struct Parser<'a> {
    fields: Vec<Field<'a>>,
}

#[allow(dead_code)]
impl<'a> Parser<'a> {
    fn builder() -> builder::ParserBuilder<'a> {
        builder::ParserBuilder::new()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Justify {
    Left,
    Right,
}

impl From<String> for Justify {
    fn from(s: String) -> Self {
        match s.to_lowercase().trim() {
            "right" => Justify::Right,
            "left" => Justify::Left,
            _ => panic!("Unknown value for Justify"),
        }
    }
}

mod builder {
    use super::*;
    use std::collections::VecDeque;
    use std::ops::Range;

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

        pub fn default_justify<T: Into<Justify>>(mut self, justify: T) -> Self {
            self.justify = justify.into();
            self
        }

        pub fn default_padding(mut self, padding: char) -> Self {
            self.padding = padding;
            self
        }

        pub fn field(self, name: &'a str) -> FieldBuilder<'a> {
            let justify = self.justify;
            let padding = self.padding;
            FieldBuilder::new(self, Some(name), justify, padding)
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

        pub fn justify<T: Into<Justify>>(mut self, justify: T) -> Self {
            self.justify = justify.into();
            self
        }

        pub fn padding(mut self, padding: char) -> Self {
            self.padding = padding;
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
        Field::_new(None, name, position, width, justify, padding)
    }

    fn _new(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_field_one() {
        let parser = Parser::builder().field("first").width(20).append().build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(
            parser.fields[0],
            Field::_new(
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
            Field::_new(
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
            Field::_new(
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
            Field::_new(
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
            Field::_new(Some(0), None, Some(5), Some(10), Justify::Left, ' ')
        );
    }
}
