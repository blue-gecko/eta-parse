use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Cursor, Lines, Read},
};

use super::*;

pub struct StringReader<'a, R: 'a> {
    r: &'a mut Reader<'a, R>,
}

impl<'a, R> StringReader<'a, R> {
    fn parse(&self, s: String) -> HashMap<String, String> {
        self.r.parser.parse(s)
    }
}

#[allow(dead_code)]
pub struct Reader<'a, R> {
    lines: Lines<BufReader<R>>,
    parser: &'a Parser<'a>,
}

#[allow(dead_code)]
impl<'a, R> Reader<'a, R>
where
    R: Read,
{
    pub fn from_reader(reader: R, parser: &'a Parser) -> Self {
        Reader {
            lines: BufReader::new(reader).lines(),
            parser,
        }
    }

    pub fn string_reader(&'a mut self) -> StringReader<'a, R> {
        StringReader { r: self }
    }
}

impl<'a, R> Iterator for StringReader<'a, R>
where
    R: Read,
{
    type Item = HashMap<String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.r.lines.next() {
            Some(Ok(s)) => Some(self.parse(s)),
            _ => None,
        }
    }
}

#[allow(dead_code)]
impl<'a> Reader<'a, File> {
    /// Creates a new reader from a filepath. Will return an io::Error if there are any issues
    /// opening the file.
    pub fn from_file(file: File, parser: &'a Parser) -> Self {
        Self::from_reader(file, parser)
    }
}

#[allow(dead_code)]
impl<'a> Reader<'a, Cursor<Vec<u8>>> {
    /// Creates a new reader from a series of bytes.
    pub fn from_bytes<T>(bytes: T, parser: &'a Parser) -> Self
    where
        T: Into<Vec<u8>>,
    {
        Self::from_reader(Cursor::new(bytes.into()), parser)
    }

    /// Creates a new reader from a `String` or `&str`.
    pub fn from_string<T>(s: T, parser: &'a Parser) -> Self
    where
        T: Into<String>,
    {
        Self::from_bytes(s.into().into_bytes(), parser)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::builder::{Buildable, Builder};
    use std::path::PathBuf;

    fn test_file<'a>(filename: &'a str) -> PathBuf {
        const TEST_DIR: &str = "./tests/data/flat/fixed/";

        let mut dir = PathBuf::from(TEST_DIR);

        dir.push(filename);
        dir
    }

    #[test]
    fn read_from_string() {
        let s = r#"1111222233334444
1111222233334444
1111222233334444"#;

        let parser = Parser::builder().field("test").range(0..4).append().build();
        let mut rdr = Reader::from_string(s, &parser);

        let rows = rdr
            .string_reader()
            .collect::<Vec<HashMap<String, String>>>();

        assert_eq!(rows.len(), 3);

        for row in rows {
            assert!(row.contains_key("test"));
            assert_eq!(row.get("test"), Some(&String::from("1111")))
        }
    }

    #[test]
    fn read_from_file() {
        let f = File::open(test_file("file-001.txt")).expect("Error reading test file");

        let parser = Parser::builder()
            .spacer(0..4)
            .field("test")
            .range(4..8)
            .append()
            .build();
        let mut rdr = Reader::from_file(f, &parser);

        let rows = rdr
            .string_reader()
            .collect::<Vec<HashMap<String, String>>>();

        assert_eq!(rows.len(), 3);

        for row in rows {
            assert!(row.contains_key("test"));
            assert_eq!(row.get("test"), Some(&String::from("2222")))
        }
    }

    #[test]
    fn read_from_unicode_string() {
        let s = r#"会げク参入せうけざ次高ぶ提宝備ず開康ネフマ制員まびぶ限下びご社近め
会げク参入せうけざ次高ぶ提宝備ず開康ネフマ制員まびぶ限下びご社近め
会げク参入せうけざ次高ぶ提宝備ず開康ネフマ制員まびぶ限下びご社近め"#;

        let parser = Parser::builder()
            .spacer(0..10)
            .field("test")
            .range(10..20)
            .append()
            .build();
        let mut rdr = Reader::from_string(s, &parser);

        let rows = rdr
            .string_reader()
            .collect::<Vec<HashMap<String, String>>>();

        assert_eq!(rows.len(), 3);

        for row in rows {
            assert!(row.contains_key("test"));
            assert_eq!(row.get("test"), Some(&String::from("高ぶ提宝備ず開康ネフ")))
        }
    }
}
