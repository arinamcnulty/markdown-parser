use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct MarkdownParser;

pub fn parse_markdown(input: &str) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    MarkdownParser::parse(Rule::document_structure, input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_paragraph() {
        let input = "Hello world";
        let result = parse_markdown(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bold_text() {
        let input = "**bold text**";
        let result = parse_markdown(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_horizontal_rule() {
        let input = "---";
        let result = parse_markdown(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_blockquote() {
        let input = "> This is a quote";
        let result = parse_markdown(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inline_link() {
        let input = "[link text](url)";
        let result = parse_markdown(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_italic_text() {
        let input = "*italic text*";
        let result = parse_markdown(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mixed_formatting() {
        let input = "**bold** and *italic*";
        let result = parse_markdown(input);
        assert!(result.is_ok());
    }
}
