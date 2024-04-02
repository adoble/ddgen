use std::ops::Range;

//use crossterm::style::Stylize;

#[derive(Copy, PartialEq, Debug, Clone)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub fn envelopes(&self, sub_span: Span) -> bool {
        sub_span.start >= self.start && sub_span.end <= self.end
    }
}

pub struct ContentSpans(Vec<Span>);
impl ContentSpans {
    fn from(contents: &str) -> ContentSpans {
        let mut spans: Vec<Span> = Vec::new();
        let mut start: usize = 1;
        let mut end: usize = 0;

        for line in contents.lines() {
            end = end + line.len() + 1; // Includes the newline
            let span = Span { start, end };
            spans.push(span);
            start = end + 1;
        }

        ContentSpans(spans)
    }

    pub fn location(&self, span: Span) -> Option<ErrorLocation> {
        for (i, r) in self.0.iter().enumerate() {
            if r.envelopes(span) {
                let location = ErrorLocation {
                    line: i + 1, // 0 based elements to 1 based lines
                    start_column: span.start - r.start + 1,
                    end_column: span.end - r.start + 1,
                };
                return Some(location);
            }
        }

        None
    }
}

// // Is the sub-range really completely contained in the range.
// fn range_is_subset(range: Range<usize>, sub_range: Range<usize>) -> bool {
//     sub_range.start >= range.start && sub_range.end <= range.start
// }

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ErrorLocation {
    line: usize,
    start_column: usize,
    end_column: usize,
}

pub fn error_report(contents: &str, message: &str, span_range: Option<Range<usize>>) {
    let span = match span_range {
        Some(range) => Span {
            start: range.start,
            end: range.end,
        },
        None => {
            println!("{}", message);
            return;
        }
    };

    let content_ranges = ContentSpans::from(contents);
    let location = content_ranges.location(span);
    match location {
        Some(loc) => println!(
            "{} at line: {}, columns: {}..{}",
            message, loc.line, loc.start_column, loc.end_column
        ),
        None => println!("{}", message),
    }

    //println!("{} {:?}", message.red(), span);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_load_content() {
        let content = "This is\nsome content\nwith many lines.";
        //                   12345678 9012345678901 2345678901234567
        //                             1         2          3

        let spans = ContentSpans::from(content);

        assert_eq!(spans.0.len(), 3);

        assert_eq!(spans.0[0], Span { start: 1, end: 8 });
        assert_eq!(spans.0[1], Span { start: 9, end: 21 });
        assert_eq!(spans.0[2], Span { start: 22, end: 38 });
    }

    #[test]
    fn test_location() {
        let content = "This is\nsome content\nwith many lines.";
        //                   12345678 9012345678901 2345678901234567
        //                             1         2          3

        let spans = ContentSpans::from(content);

        let search_range = Span { start: 12, end: 20 };
        let location = spans.location(search_range).unwrap();

        assert_eq!(
            location,
            ErrorLocation {
                line: 2,
                start_column: 4,
                end_column: 12
            }
        );
    }

    #[test]
    fn test_location_not_found() {
        let content = "This is\nsome content\nwith many lines.";
        //                   12345678 9012345678901 2345678901234567
        //                             1         2          3

        let spans = ContentSpans::from(content);

        let search_range = Span { start: 30, end: 39 };
        assert!(spans.location(search_range).is_none());

        let search_range = Span { start: 40, end: 50 };
        assert!(spans.location(search_range).is_none());
    }

    #[test]
    fn test_location_spans_lines() {
        let content = "This is\nsome content\nwith many lines.";
        //                   12345678 9012345678901 2345678901234567
        //                             1         2          3

        let spans = ContentSpans::from(content);

        let search_range = Span { start: 18, end: 24 };
        assert!(spans.location(search_range).is_none());
    }

    #[test]
    fn test_sub_range() {
        let span = Span { start: 4, end: 9 };
        let sub_span = Span { start: 5, end: 7 };

        assert!(span.envelopes(sub_span));
    }
}
