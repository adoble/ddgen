use std::ops::Range;

use crossterm::style::Stylize;

pub struct ContentRanges(Vec<Range<usize>>);
impl ContentRanges {
    fn from(contents: &str) -> ContentRanges {
        let mut ranges: Vec<Range<usize>> = Vec::new();
        let mut count = 0;
        let mut start: usize = 1;
        let mut end: usize = 0;

        for line in contents.lines() {
            end = end + line.len() + 1; // Includes the newline
            let range = Range { start, end };
            ranges.push(range);
            start = end + 1;
        }

        ContentRanges(ranges)
    }
}
pub struct ErrorLocation {
    line: usize,
    start_column: usize,
    end_column: usize,
}

pub fn error_report(contents: &str, message: &str, span: Option<Range<usize>>) {
    let mut content_ranges = ContentRanges::from(contents);

    match span {
        Some(span) => {
            let location = find_location(span);
            println!(
                "{} at line: {}, columns: {}..{}",
                message, location.line, location.start_column, location.end_column
            );
        }
        None => println!("{}", message),
    }

    // let location = find_location(span);

    //println!("{} {:?}", message.red(), span);
}

fn find_location(span: Range<usize>) -> ErrorLocation {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_load_content() {
        let content = "This is\nsome content\nwith many lines.";
        //                   12345678 9012345678901 2345678901234567
        //                             1         2          3

        let ranges = ContentRanges::from(content);

        assert_eq!(ranges.0.len(), 3);

        assert_eq!(ranges.0[0], Range { start: 1, end: 8 });
        assert_eq!(ranges.0[1], Range { start: 9, end: 21 });
        assert_eq!(ranges.0[2], Range { start: 22, end: 38 });
    }
}
