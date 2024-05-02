/// Generates a document comment from a multi-line string.
///
/// Note: This is to correct an apparent deficiency in the `gengo` crate
/// in that it generates doc comment using the `#[doc]` attributes, rather
/// than the, more usual, "///" prefix.  
#[derive(PartialEq, Debug)]
pub struct DocComment(Vec<String>);

impl DocComment {
    /// Creates a doc comment based on the string by inserting
    /// "///" before each line.  
    pub fn from_string(comment: &str) -> DocComment {
        let doc_comment_lines = comment
            .lines()
            .map(|s| {
                let mut line = s.to_string();
                line.insert_str(0, "/// ");
                line
            })
            .collect();

        DocComment(doc_comment_lines)
    }

    /// Returns an empty doc comment "///"
    pub fn empty() -> String {
        "///".to_string()
    }

    /// Gets the doc comment as a single String
    pub fn as_string(&self) -> String {
        self.0.join("\n")
    }

    /// Gets a iterator over all the doc comment lines.
    #[allow(dead_code)]
    pub fn lines(&self) -> std::slice::Iter<'_, std::string::String> {
        self.0.iter()
    }

    #[allow(dead_code)]
    pub fn number_lines(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion() {
        let s = "This is a comment";

        let d = DocComment::from_string(&s.to_string());

        assert_eq!(d.as_string(), "/// This is a comment");
    }

    #[test]
    fn conversion_multi_line() {
        let s = "This is a comment\nAnother line.\nFinal line.";
        let expected = "/// This is a comment\n/// Another line.\n/// Final line.";

        let d = DocComment::from_string(&s.to_string());

        assert_eq!(d.as_string(), expected);
    }

    #[test]
    fn iterate() {
        let s = "This is a comment.\nA new lne of comment.\nMore comments.";
        let expected = [
            &"/// This is a comment.".to_string(),
            &"/// A new lne of comment.".to_string(),
            &"/// More comments.".to_string(),
        ];

        let doc_comment = DocComment::from_string(&s.to_string());

        let v: Vec<&String> = doc_comment.lines().collect();

        assert_eq!(v, expected);
    }
}
