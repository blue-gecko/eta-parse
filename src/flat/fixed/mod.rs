use std::ops::{Bound, Bound::Excluded, Bound::Included, Bound::Unbounded, Range};

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

mod builder {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Debug)]
    pub struct ParserBuilder<'a> {
        fields: VecDeque<Field<'a>>,
    }

    #[allow(dead_code)]
    impl<'a> ParserBuilder<'a> {
        pub fn new() -> Self {
            ParserBuilder {
                fields: VecDeque::new(),
            }
        }

        pub fn add_width(&mut self, id: &'a str, width: u32) -> &mut Self {
            self.fields.push_back(Field::new(0, id).with_width(width));
            self
        }

        pub fn add_position(&mut self, id: &'a str, position: u32) -> &mut Self {
            self.fields
                .push_back(Field::new(0, id).with_position(position));
            self
        }

        pub fn add_range(&mut self, id: &'a str, start: u32, end: u32) -> &mut Self {
            self.fields
                .push_back(Field::new(0, id).with_range(start..end));
            self
        }

        pub fn add(&mut self, id: &'a str, position: u32, width: u32) -> &mut Self {
            self.fields
                .push_back(Field::new(0, id).with_position(position).with_width(width));
            self
        }

        pub fn build(&mut self) -> Parser<'a> {
            let mut fields: Vec<Field> = Vec::new();
            let mut position: u32 = 0;
            let mut index = 0;
            while let Some(mut current) = self.fields.pop_front() {
                current.index = index;
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
    }
}

#[derive(Debug)]
struct Field<'a> {
    index: u32,
    id: &'a str,
    position: Option<u32>,
    width: Option<u32>,
}

#[allow(dead_code)]
impl<'a> Field<'a> {
    fn new(index: u32, id: &'a str) -> Self {
        Field {
            index,
            id,
            position: None,
            width: None,
        }
    }

    pub fn field(id: &'a str) -> Self {
        Field::new(0, id)
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_position(mut self, position: u32) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_range(mut self, range: Range<u32>) -> Self {
        self.position = Some(range.start);
        self.width = Some(range.end - range.start);
        self
    }

    fn get_bounds(&self) -> (Bound<u32>, Bound<u32>) {
        match (self.position, self.width) {
            (Some(position), Some(width)) => (Included(position), Excluded(position + width)),
            (Some(position), None) => (Included(position), Unbounded),
            (None, Some(width)) => (Unbounded, Excluded(width)),
            (None, None) => (Unbounded, Unbounded),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_field_default() {
        let field = Field::new(1, "first");
        assert_eq!(field.index, 1);
        assert_eq!(field.id, "first");
        assert_eq!(field.position, None);
        assert_eq!(field.width, None);
        assert_eq!(field.get_bounds(), (Unbounded, Unbounded));
    }

    #[test]
    fn check_field_with_width() {
        let field = Field::new(1, "first").with_width(10);
        assert_eq!(field.index, 1);
        assert_eq!(field.id, "first");
        assert_eq!(field.position, None);
        assert_eq!(field.width, Some(10));
        assert_eq!(field.get_bounds(), (Unbounded, Excluded(10)));
    }

    #[test]
    fn check_field_with_position() {
        let field = Field::new(1, "first").with_position(12);
        assert_eq!(field.index, 1);
        assert_eq!(field.id, "first");
        assert_eq!(field.position, Some(12));
        assert_eq!(field.width, None);
        assert_eq!(field.get_bounds(), (Included(12), Unbounded));
    }

    #[test]
    fn check_field_with_both() {
        let field = Field::new(1, "first").with_position(12).with_width(10);
        assert_eq!(field.index, 1);
        assert_eq!(field.id, "first");
        assert_eq!(field.position, Some(12));
        assert_eq!(field.width, Some(10));
        assert_eq!(field.get_bounds(), (Included(12), Excluded(22)));
    }

    #[test]
    fn check_field_with_range() {
        let field = Field::new(1, "first").with_position(12).with_range(10..20);
        assert_eq!(field.index, 1);
        assert_eq!(field.id, "first");
        assert_eq!(field.position, Some(10));
        assert_eq!(field.width, Some(10));
    }

    #[test]
    fn check_build_empty() {
        let parser = Parser::builder().build();

        assert_eq!(parser.fields.len(), 0);
    }

    #[test]
    fn check_build_one() {
        let parser = Parser::builder().add("first", 0, 10).build();

        assert_eq!(parser.fields.len(), 1);
        assert_eq!(parser.fields[0].id, "first");
        assert_eq!(parser.fields[0].position, Some(0));
        assert_eq!(parser.fields[0].width, Some(10));
    }

    #[test]
    fn check_build_width() {
        let parser = Parser::builder()
            .add_width("first", 12)
            .add_width("second", 14)
            .build();

        assert_eq!(parser.fields.len(), 2);
        assert_eq!(parser.fields[0].id, "first");
        assert_eq!(parser.fields[0].position, Some(0));
        assert_eq!(parser.fields[0].width, Some(12));
        assert_eq!(parser.fields[1].id, "second");
        assert_eq!(parser.fields[1].position, Some(12));
        assert_eq!(parser.fields[1].width, Some(14));
    }

    #[test]
    fn check_build_position() {
        let parser = Parser::builder()
            .add_position("first", 0)
            .add_position("second", 12)
            .build();

        assert_eq!(parser.fields.len(), 2);
        assert_eq!(parser.fields[0].id, "first");
        assert_eq!(parser.fields[0].position, Some(0));
        assert_eq!(parser.fields[0].width, Some(12));
        assert_eq!(parser.fields[1].id, "second");
        assert_eq!(parser.fields[1].position, Some(12));
        assert_eq!(parser.fields[1].width, None);
    }
}
