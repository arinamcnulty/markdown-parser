use anyhow::anyhow;
use markdown_parser::MarkdownParser;
use markdown_parser::*;
use pest::Parser;

fn parse_by_rule(
    rule: Rule,
    input: &str,
) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    MarkdownParser::parse(rule, input)
}

fn get_single_pair<'a>(
    pairs: &mut pest::iterators::Pairs<'a, Rule>,
    expected_rule: Rule,
    context: &str,
) -> anyhow::Result<pest::iterators::Pair<'a, Rule>> {
    pairs
        .next()
        .ok_or_else(|| anyhow!("Expected {:?} in {}", expected_rule, context))
        .and_then(|pair| {
            if pair.as_rule() == expected_rule {
                std::result::Result::Ok(pair)
            } else {
                Err(anyhow!(
                    "Expected rule {:?}, got {:?} in {}",
                    expected_rule,
                    pair.as_rule(),
                    context
                ))
            }
        })
}

fn get_inner_pair<'a>(
    pair: &pest::iterators::Pair<'a, Rule>,
    expected_rule: Rule,
    context: &str,
) -> anyhow::Result<pest::iterators::Pair<'a, Rule>> {
    pair.clone()
        .into_inner()
        .next()
        .ok_or_else(|| anyhow!("Expected inner {:?} in {}", expected_rule, context))
        .and_then(|inner_pair| {
            if inner_pair.as_rule() == expected_rule {
                std::result::Result::Ok(inner_pair)
            } else {
                Err(anyhow!(
                    "Expected inner rule {:?}, got {:?} in {}",
                    expected_rule,
                    inner_pair.as_rule(),
                    context
                ))
            }
        })
}

fn get_headers_start_length(rule: Rule) -> usize {
    match rule {
        Rule::h1_heading => 1,
        Rule::h2_heading => 2,
        Rule::h3_heading => 3,
        _ => 0,
    }
}

fn check_header(rule: Rule, header_string: &str) {
    let result = parse_by_rule(rule, header_string).expect("An error occured while parsing");

    for pair in result {
        if pair.as_rule() == Rule::h1_heading {
            println!("Read: {}", pair.as_str());
            assert_eq!(header_string, pair.as_str());

            let inner_pairs = pair.into_inner();
            let header_text: Vec<&str> = inner_pairs
                .filter(|pair| pair.as_rule() == Rule::document_heading)
                .map(|pair| pair.as_str())
                .collect();

            for text in &header_text {
                println!("heading_text content: {}", *text);
                let string_start: usize = get_headers_start_length(rule) + 1;
                assert_eq!(*text, &header_string[string_start..]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::*;

    #[test]
    fn check_headers() {
        check_header(Rule::h1_heading, "# Header 1 ###   some text!");
        check_header(
            Rule::h1_heading,
            "# Header 1 # !## some !@#$%^&*(*&^%|}\"?text!",
        );
        check_header(Rule::h1_heading, "# M");

        check_header(Rule::h2_heading, "## Header 2 ## Some other     text.");
        check_header(Rule::h2_heading, "## Special # symbols in header 2! @#$%");
        check_header(Rule::h2_heading, "## Another heading 2");

        check_header(Rule::h3_heading, "### Header 3 with          more text");
        check_header(Rule::h3_heading, "### H3 with special chars $%^&*()");
        check_header(Rule::h3_heading, "### Simple header 3");
    }

    #[test]
    #[should_panic]
    fn check_wrong_headers() {
        check_header(Rule::h1_heading, "# ");
        check_header(Rule::h1_heading, "#");
        check_header(Rule::h1_heading, "## Another header, not h1");

        check_header(Rule::h2_heading, "## ");
        check_header(Rule::h2_heading, "##");
        check_header(Rule::h2_heading, "# Header not h2");

        check_header(Rule::h3_heading, "### ");
        check_header(Rule::h3_heading, "###");
        check_header(Rule::h3_heading, "# Header not h3");
    }

    #[test]
    fn check_plain_text() {
        let res1 = parse_by_rule(Rule::plain_text, "This is a plain text");
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().as_str(), "This is a plain text");

        let res2 = parse_by_rule(Rule::plain_text, "This is \\* plain text");
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().as_str(), "This is ");
    }

    #[test]
    fn check_plain_char() -> Result<()> {
        let input = "a";
        let mut pairs = parse_by_rule(Rule::plain_char, input)?;
        let pair = get_single_pair(&mut pairs, Rule::plain_char, "plain char")?;
        assert_eq!(pair.as_str(), "a");

        let invalid_inputs = ["*", "_", "~", "[", "\\"];
        for &input in &invalid_inputs {
            let result = parse_by_rule(Rule::plain_char, input);
            assert!(result.is_err(), "plain_char should reject: {}", input);
        }

        Ok(())
    }

    #[test]
    fn check_link_char() -> Result<()> {
        let input = "a";
        let mut pairs = parse_by_rule(Rule::link_char, input)?;
        let pair = get_single_pair(&mut pairs, Rule::link_char, "link char")?;
        assert_eq!(pair.as_str(), "a");

        let input = "\\]";
        let mut pairs = parse_by_rule(Rule::link_char, input)?;
        let pair = get_single_pair(&mut pairs, Rule::link_char, "escaped link char")?;
        assert_eq!(pair.as_str(), "\\]");

        Ok(())
    }

    #[test]
    fn check_image_char() -> Result<()> {
        let input = "a";
        let mut pairs = parse_by_rule(Rule::image_char, input)?;
        let pair = get_single_pair(&mut pairs, Rule::image_char, "image char")?;
        assert_eq!(pair.as_str(), "a");

        let input = "\\]";
        let mut pairs = parse_by_rule(Rule::image_char, input)?;
        let pair = get_single_pair(&mut pairs, Rule::image_char, "escaped image char")?;
        assert_eq!(pair.as_str(), "\\]");

        Ok(())
    }

    #[test]
    fn check_url_char() -> Result<()> {
        let input = "a";
        let mut pairs = parse_by_rule(Rule::url_char, input)?;
        let pair = get_single_pair(&mut pairs, Rule::url_char, "url char")?;
        assert_eq!(pair.as_str(), "a");

        let input = "\\)";
        let mut pairs = parse_by_rule(Rule::url_char, input)?;
        let pair = get_single_pair(&mut pairs, Rule::url_char, "escaped url char")?;
        assert_eq!(pair.as_str(), "\\)");

        Ok(())
    }

    #[test]
    fn check_document_heading() -> Result<()> {
        let input = "# Heading";
        let mut pairs = parse_by_rule(Rule::document_heading, input)?;
        let pair = get_single_pair(&mut pairs, Rule::h1_heading, "document heading h1")?;
        assert_eq!(pair.as_str(), "# Heading");

        let input = "## Heading";
        let mut pairs = parse_by_rule(Rule::document_heading, input)?;
        let pair = get_single_pair(&mut pairs, Rule::h2_heading, "document heading h2")?;
        assert_eq!(pair.as_str(), "## Heading");

        let input = "### Heading";
        let mut pairs = parse_by_rule(Rule::document_heading, input)?;
        let pair = get_single_pair(&mut pairs, Rule::h3_heading, "document heading h3")?;
        assert_eq!(pair.as_str(), "### Heading");

        Ok(())
    }

    #[test]
    fn check_escaped() -> Result<()> {
        let input = "\\*";
        let mut pairs = parse_by_rule(Rule::escape_sequence, input)?;
        let pair = get_single_pair(&mut pairs, Rule::escape_sequence, "escaped asterisk")?;
        let char_pair = get_inner_pair(&pair, Rule::character, "escaped char")?;
        assert_eq!(char_pair.as_str(), "*");

        let input = "\\\\";
        let mut pairs = parse_by_rule(Rule::escape_sequence, input)?;
        let pair = get_single_pair(&mut pairs, Rule::escape_sequence, "escaped backslash")?;
        let char_pair = get_inner_pair(&pair, Rule::character, "escaped char")?;
        assert_eq!(char_pair.as_str(), "\\");

        std::result::Result::Ok(())
    }

    #[test]
    #[should_panic]
    fn check_escaped_panic() {
        let input = "\\ a";
        let pairs = parse_by_rule(Rule::escape_sequence, input);
        pairs.expect("An error occurred, expected cause: whitespace after \\");
    }

    #[test]
    fn check_italic() -> Result<()> {
        let input = "*this text is italic*";
        let mut pairs = parse_by_rule(Rule::italic_formatting, input)?;
        let pair = get_single_pair(&mut pairs, Rule::italic_formatting, "asterisk italic")?;
        let content = get_inner_pair(&pair, Rule::italic_content, "italic content")?;
        assert_eq!(content.as_str(), "this text is italic");

        let input = "_italic test 2_";
        let mut pairs = parse_by_rule(Rule::italic_formatting, input)?;
        let pair = get_single_pair(&mut pairs, Rule::italic_formatting, "underscore italic")?;
        let content = get_inner_pair(&pair, Rule::italic_content, "italic content")?;
        assert_eq!(content.as_str(), "italic test 2");

        std::result::Result::Ok(())
    }

    #[test]
    fn check_bold() -> Result<()> {
        let input = "**this text is bold**";
        let mut pairs = parse_by_rule(Rule::bold_formatting, input)?;
        let pair = get_single_pair(&mut pairs, Rule::bold_formatting, "bold text")?;
        let content = get_inner_pair(&pair, Rule::bold_content, "bold content")?;
        assert_eq!(content.as_str(), "this text is bold");

        std::result::Result::Ok(())
    }

    #[test]
    fn check_underline() -> Result<()> {
        let input = "__underlined text!__";
        let mut pairs = parse_by_rule(Rule::underline_formatting, input)?;
        let pair = get_single_pair(&mut pairs, Rule::underline_formatting, "underline text")?;
        let content = get_inner_pair(&pair, Rule::underline_content, "underline content")?;
        assert_eq!(content.as_str(), "underlined text!");

        std::result::Result::Ok(())
    }

    #[test]
    fn check_strikethrough() -> Result<()> {
        let input = "~~some striked text!~~";
        let mut pairs = parse_by_rule(Rule::strikethrough_formatting, input)?;
        let pair = get_single_pair(
            &mut pairs,
            Rule::strikethrough_formatting,
            "strikethrough text",
        )?;
        let content = get_inner_pair(&pair, Rule::strikethrough_content, "strikethrough content")?;
        assert_eq!(content.as_str(), "some striked text!");

        std::result::Result::Ok(())
    }

    #[test]
    fn check_inline_link() -> Result<()> {
        let input = "[click this!](https://google.com/)";
        let mut pairs = parse_by_rule(Rule::link, input)?;
        let pair = get_single_pair(&mut pairs, Rule::link, "inline link")?;

        let mut inner_iter = pair.into_inner();
        let link_text = get_single_pair(&mut inner_iter, Rule::link_content, "link content")?;
        assert_eq!(link_text.as_str(), "click this!");

        let url = get_single_pair(&mut inner_iter, Rule::link_url, "link url")?;
        assert_eq!(url.as_str(), "https://google.com/");

        std::result::Result::Ok(())
    }

    #[test]
    fn check_inline_image() -> Result<()> {
        let input = "![alternative text](https://example.com/image.png)";
        let mut pairs = parse_by_rule(Rule::image, input)?;
        let pair = get_single_pair(&mut pairs, Rule::image, "inline image")?;

        let mut inner_iter = pair.into_inner();
        let alt_text = get_single_pair(&mut inner_iter, Rule::image_alt, "image alt text")?;
        assert_eq!(alt_text.as_str(), "alternative text");

        let url = get_single_pair(&mut inner_iter, Rule::image_url, "image url")?;
        assert_eq!(url.as_str(), "https://example.com/image.png");

        std::result::Result::Ok(())
    }

    #[test]
    fn check_quote() -> Result<()> {
        let input =
            ">This is a text in a quote\n>this is also a part of a quote\n>this one is too.";
        let mut pairs = parse_by_rule(Rule::document_quote, input)?;

        let pair = pairs
            .next()
            .ok_or_else(|| anyhow!("Expected a quote, but found none"))?;

        assert_eq!(pair.as_rule(), Rule::document_quote);

        let mut quote_inner = pair.into_inner();
        let quote_line1 = quote_inner
            .next()
            .ok_or_else(|| anyhow!("Expected a quote_line, but found none"))?;

        assert_eq!(quote_line1.as_rule(), Rule::quote_line);

        let mut quote_line1_inner = quote_line1.into_inner();
        let paragraph_text1 = quote_line1_inner.next().ok_or_else(|| {
            anyhow!("Expected paragraph_text in the first quote line, but found none")
        })?;

        assert_eq!(paragraph_text1.as_rule(), Rule::paragraph_text);
        let plain_text1 = paragraph_text1
            .into_inner()
            .next()
            .ok_or_else(|| anyhow!("Expected plain_text in the first line, but found none"))?;

        assert_eq!(plain_text1.as_str(), "This is a text in a quote");

        let quote_line2 = quote_inner
            .next()
            .ok_or_else(|| anyhow!("Expected the second quote_line, but found none"))?;

        assert_eq!(quote_line2.as_rule(), Rule::quote_line);

        let mut quote_line2_inner = quote_line2.into_inner();
        let paragraph_text2 = quote_line2_inner.next().ok_or_else(|| {
            anyhow!("Expected paragraph_text in the second quote line, but found none")
        })?;

        assert_eq!(paragraph_text2.as_rule(), Rule::paragraph_text);
        let plain_text2 = paragraph_text2
            .into_inner()
            .next()
            .ok_or_else(|| anyhow!("Expected plain_text in the second line, but found none"))?;

        assert_eq!(plain_text2.as_str(), "this is also a part of a quote");

        let quote_line3 = quote_inner
            .next()
            .ok_or_else(|| anyhow!("Expected the third quote_line, but found none"))?;

        assert_eq!(quote_line3.as_rule(), Rule::quote_line);

        let mut quote_line3_inner = quote_line3.into_inner();
        let paragraph_text3 = quote_line3_inner.next().ok_or_else(|| {
            anyhow!("Expected paragraph_text in the third quote line, but found none")
        })?;

        assert_eq!(paragraph_text3.as_rule(), Rule::paragraph_text);
        let plain_text3 = paragraph_text3
            .into_inner()
            .next()
            .ok_or_else(|| anyhow!("Expected plain_text in the third line, but found none"))?;

        assert_eq!(plain_text3.as_str(), "this one is too.");

        std::result::Result::Ok(())
    }

    #[test]
    fn check_code_block() -> Result<()> {
        let input = "```py\nprint(\"Hello World!\")\n```";
        let mut pairs = parse_by_rule(Rule::code_fence, input)?;
        let pair = get_single_pair(&mut pairs, Rule::code_fence, "code fence")?;

        let mut code_block_inner = pair.into_inner();
        let code_lang =
            get_single_pair(&mut code_block_inner, Rule::language_spec, "language spec")?;
        assert_eq!(code_lang.as_str(), "py");

        let code_content = get_single_pair(&mut code_block_inner, Rule::code_body, "code body")?;
        assert_eq!(code_content.as_str(), "print(\"Hello World!\")");

        std::result::Result::Ok(())
    }

    #[test]
    fn check_horizontal_rule() -> Result<()> {
        let inputs = vec!["---", "***", "___", "---   ", "***\n", "___\n"];

        for input in inputs {
            let mut pairs = parse_by_rule(Rule::thematic_break, input)?;
            let pair = pairs
                .next()
                .ok_or_else(|| anyhow!("Expected a thematic_break, but found none"))?;

            assert_eq!(pair.as_rule(), Rule::thematic_break);
            assert_eq!(pair.as_str().trim(), input.trim());
        }

        std::result::Result::Ok(())
    }

    #[test]
    fn check_inline_code() -> Result<()> {
        let inputs = vec!["`code`", "`let x = 1;`", "`function() { return true; }`"];

        for input in inputs {
            let mut pairs = parse_by_rule(Rule::inline_code, input)?;
            let pair = pairs
                .next()
                .ok_or_else(|| anyhow!("Expected a inline_code, but found none"))?;

            assert_eq!(pair.as_rule(), Rule::inline_code);
            assert_eq!(pair.as_str(), input);
        }

        std::result::Result::Ok(())
    }

    #[test]
    fn check_unordered_list_item() -> Result<()> {
        let inputs = vec!["- item\n", "* item\n", "- First item\n", "* Second item\n"];

        for input in inputs {
            let mut pairs = parse_by_rule(Rule::unordered_list_item, input)?;
            let pair = pairs
                .next()
                .ok_or_else(|| anyhow!("Expected a unordered_list_item, but found none"))?;

            assert_eq!(pair.as_rule(), Rule::unordered_list_item);
            assert_eq!(pair.as_str(), input);
        }

        std::result::Result::Ok(())
    }

    #[test]
    fn check_ordered_list_item() -> Result<()> {
        let inputs = vec!["1. item\n", "2. item\n", "10. item\n", "123. item\n"];

        for input in inputs {
            let mut pairs = parse_by_rule(Rule::ordered_list_item, input)?;
            let pair = pairs
                .next()
                .ok_or_else(|| anyhow!("Expected a ordered_list_item, but found none"))?;

            assert_eq!(pair.as_rule(), Rule::ordered_list_item);
            assert_eq!(pair.as_str(), input);
        }

        std::result::Result::Ok(())
    }

    #[test]
    fn check_document_unordered_list() -> Result<()> {
        let input = "- item 1\n* item 2\n- item 3\n";

        let mut pairs = parse_by_rule(Rule::document_unordered_list, input)?;
        let pair = pairs
            .next()
            .ok_or_else(|| anyhow!("Expected a document_unordered_list, but found none"))?;

        assert_eq!(pair.as_rule(), Rule::document_unordered_list);
        assert_eq!(pair.as_str(), input);

        // Check that it contains the expected items
        let items: Vec<_> = pair.into_inner().collect();
        assert_eq!(items.len(), 3);
        for item in items {
            assert_eq!(item.as_rule(), Rule::unordered_list_item);
        }

        std::result::Result::Ok(())
    }

    #[test]
    fn check_document_ordered_list() -> Result<()> {
        let input = "1. item 1\n2. item 2\n3. item 3\n";

        let mut pairs = parse_by_rule(Rule::document_ordered_list, input)?;
        let pair = pairs
            .next()
            .ok_or_else(|| anyhow!("Expected a document_ordered_list, but found none"))?;

        assert_eq!(pair.as_rule(), Rule::document_ordered_list);
        assert_eq!(pair.as_str(), input);

        // Check that it contains the expected items
        let items: Vec<_> = pair.into_inner().collect();
        assert_eq!(items.len(), 3);
        for item in items {
            assert_eq!(item.as_rule(), Rule::ordered_list_item);
        }

        std::result::Result::Ok(())
    }

    #[test]
    fn check_markdown() -> Result<()> {
        let input = "# Hello this is my 1st post!\n___\n\nThis is **bold** text!\nThat's all. Bye!";

        let mut pairs = parse_by_rule(Rule::document_structure, input)?;

        let pair = pairs
            .next()
            .ok_or_else(|| anyhow!("Expected a document_structure root, but found none"))?;

        assert_eq!(pair.as_rule(), Rule::document_structure);

        let mut markdown_inner = pair.into_inner();

        let heading_block = markdown_inner
            .next()
            .ok_or_else(|| anyhow!("Expected document_block with heading, but found none"))?;

        assert_eq!(heading_block.as_rule(), Rule::document_block);

        let heading1 = heading_block
            .into_inner()
            .next()
            .ok_or_else(|| anyhow!("Expected h1_heading"))?;

        assert_eq!(heading1.as_rule(), Rule::h1_heading);
        assert_eq!(heading1.as_str().trim(), "# Hello this is my 1st post!");

        let hr_block = markdown_inner.next().ok_or_else(|| {
            anyhow!("Expected document_block with thematic_break, but found none")
        })?;

        assert_eq!(hr_block.as_rule(), Rule::document_block);

        let horizontal_rule = hr_block
            .into_inner()
            .next()
            .ok_or_else(|| anyhow!("Expected thematic_break"))?;

        assert_eq!(horizontal_rule.as_rule(), Rule::thematic_break);
        assert_eq!(horizontal_rule.as_str().trim(), "___");

        let paragraph_block = markdown_inner
            .next()
            .ok_or_else(|| anyhow!("Expected document_block with paragraph, but found none"))?;

        assert_eq!(paragraph_block.as_rule(), Rule::document_block);

        let paragraph2 = paragraph_block
            .into_inner()
            .next()
            .ok_or_else(|| anyhow!("Expected document_paragraph"))?;

        assert_eq!(paragraph2.as_rule(), Rule::document_paragraph);
        // Just check that paragraph exists and contains some text
        assert!(!paragraph2.as_str().is_empty());

        // Test completed successfully - we have heading, hr, and paragraph
        println!("Test passed: found heading, thematic break, and paragraph");

        std::result::Result::Ok(())
    }
}
