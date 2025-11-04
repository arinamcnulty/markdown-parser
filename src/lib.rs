//! # Markdown Parser Library
//!
//! This library provides comprehensive Markdown parsing capabilities with HTML conversion.
//! It uses Pest grammar for efficient parsing and supports various Markdown elements.
//!
//! ## Features
//!
//! - Full Markdown syntax support (headings, paragraphs, links, images, formatting)
//! - Robust error handling with custom error types
//! - HTML output generation with proper escaping
//! - Command-line interface integration
//!
//! ## Example
//!
//! ```rust
//! use markdown_parser::{parse_markdown, str_to_html};
//!
//! let markdown = "# Hello World\n\nThis is **bold** text.";
//! let html = str_to_html(markdown).unwrap();
//! println!("{}", html.join("\n"));
//! ```

use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
};

use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;

/// Custom error type for markdown parsing operations.
/// Provides detailed error information for different failure scenarios.
#[derive(Debug, thiserror::Error)]
pub enum MarkdownError {
    #[error("Parsing failed: {0}")]
    ParseError(String),

    #[error("File operation failed: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct MarkdownParser;

/// Main parsing function that processes markdown input.
/// Returns parsed syntax tree or error if parsing fails.
///
/// # Arguments
/// * `input` - Raw markdown text as string slice
///
/// # Returns
/// Result containing parsed pairs or MarkdownError
pub fn parse_markdown(input: &str) -> Result<Pairs<'_, Rule>, MarkdownError> {
    MarkdownParser::parse(Rule::document_structure, input)
        .map_err(|e| MarkdownError::ParseError(e.to_string()))
}


/// Convert markdown string to vector of HTML strings.
/// Each element represents one HTML line/tag.
///
/// # Arguments
/// * `input` - Markdown text to convert
///
/// # Returns
/// Vector of HTML strings or MarkdownError
pub fn str_to_html(input: &str) -> Result<Vec<String>, MarkdownError> {
    let mut parsed = parse_markdown(input)?;
    let document = parsed
        .next()
        .ok_or_else(|| MarkdownError::ParseError("Empty document".to_string()))?;

    let results: Result<Vec<String>, MarkdownError> = document
        .into_inner()
        .filter(|pair| !matches!(pair.as_rule(), Rule::EOI))
        .map(convert_to_html)
        .collect();

    results
}

/// Convert a single parsed rule to HTML representation.
/// This is the core conversion dispatcher for different markdown elements.
///
/// # Arguments
/// * `pair` - Pest Pair representing parsed rule
///
/// # Returns
/// HTML string or MarkdownError
fn convert_to_html(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    match pair.as_rule() {
        Rule::document_block => {
            let inner = pair.into_inner().next().unwrap();
            convert_to_html(inner)
        }
        Rule::document_heading => process_document_heading(pair),
        Rule::h1_heading | Rule::h2_heading | Rule::h3_heading => process_heading(pair),
        Rule::document_paragraph => process_document_paragraph(pair),
        Rule::document_quote => process_document_quote(pair),
        Rule::quote_line => process_quote_line(pair),
        Rule::paragraph_text => process_paragraph_text(pair),
        Rule::document_unordered_list => process_unordered_list(pair),
        Rule::document_ordered_list => process_ordered_list(pair),
        Rule::unordered_list_item => process_list_item(pair),
        Rule::ordered_list_item => process_list_item(pair),
        Rule::code_fence => process_code_fence(pair),
        Rule::thematic_break => Ok("<hr>".to_string()),
        Rule::blank_line => Ok("<br>".to_string()),
        Rule::EOI => Ok(String::new()),
        _ => Err(MarkdownError::ParseError(format!(
            "Unknown rule: {:?}",
            pair.as_rule()
        ))),
    }
}

/// Process document heading container.
fn process_document_heading(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    let inner = pair.into_inner().next().unwrap();
    process_heading(inner)
}

/// Process heading elements (H1, H2, H3).
fn process_heading(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    let level = match pair.as_rule() {
        Rule::h1_heading => 1,
        Rule::h2_heading => 2,
        Rule::h3_heading => 3,
        _ => return Err(MarkdownError::ParseError("Invalid heading".to_string())),
    };

    let content = pair.as_str();
    let text = content
        .trim_start_matches('#')
        .trim_start_matches(char::is_whitespace)
        .trim_end_matches('\n')
        .trim();

    Ok(format!("<h{level}>{}</h{level}>", html_escape::encode_text(text)))
}

fn process_document_paragraph(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    process_paragraph(pair)
}

fn process_paragraph(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    let content: Result<String, MarkdownError> = pair
        .into_inner()
        .map(|line| process_paragraph_line(line))
        .collect();

    Ok(format!("<p>{}</p>", content?))
}

fn process_paragraph_text(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    pair.into_inner()
        .map(|inline| process_inline_element(inline))
        .collect()
}

fn process_paragraph_line(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    pair.into_inner()
        .map(|inline| process_inline_element(inline))
        .collect()
}

/// Process inline elements (text, formatting, links, images).
fn process_inline_element(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    match pair.as_rule() {
        Rule::plain_text => Ok(html_escape::encode_text(pair.as_str()).to_string()),
        Rule::inline_code => {
            let full = pair.as_str();
            let code = full.strip_prefix('`').and_then(|s| s.strip_suffix('`')).unwrap_or("");
            Ok(format!("<code>{}</code>", html_escape::encode_text(code)))
        }
        Rule::link => process_link(pair),
        Rule::image => process_image(pair),
        Rule::bold_formatting => {
            let content = process_bold_content(pair)?;
            Ok(format!("<strong>{content}</strong>"))
        }
        Rule::italic_formatting => {
            let content = process_italic_content(pair)?;
            Ok(format!("<em>{content}</em>"))
        }
        Rule::strikethrough_formatting => {
            let content = process_strikethrough_content(pair)?;
            Ok(format!("<del>{content}</del>"))
        }
        Rule::underline_formatting => {
            let content = process_underline_content(pair)?;
            Ok(format!("<u>{content}</u>"))
        }
        Rule::text_formatting => process_text_formatting(pair),
        Rule::escape_sequence => process_escape_sequence(pair),
        _ => Ok(html_escape::encode_text(pair.as_str()).to_string()),
    }
}

/// Process text formatting (bold, italic, strikethrough, underline).
fn process_text_formatting(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    let rule = pair.as_rule();
    match rule {
        Rule::bold_formatting => {
            let content = process_bold_content(pair)?;
            Ok(format!("<strong>{content}</strong>"))
        }
        Rule::italic_formatting => {
            let content = process_italic_content(pair)?;
            Ok(format!("<em>{content}</em>"))
        }
        Rule::strikethrough_formatting => {
            let content = process_strikethrough_content(pair)?;
            Ok(format!("<del>{content}</del>"))
        }
        Rule::underline_formatting => {
            let content = process_underline_content(pair)?;
            Ok(format!("<u>{content}</u>"))
        }
        _ => Ok(html_escape::encode_text(pair.as_str()).to_string()),
    }
}

fn process_bold_content(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    pair.into_inner()
        .next()
        .map(|p| html_escape::encode_text(p.as_str()).to_string())
        .ok_or_else(|| MarkdownError::ParseError("Empty bold content".to_string()))
}

fn process_italic_content(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    pair.into_inner()
        .next()
        .map(|p| html_escape::encode_text(p.as_str()).to_string())
        .ok_or_else(|| MarkdownError::ParseError("Empty italic content".to_string()))
}

fn process_strikethrough_content(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    pair.into_inner()
        .next()
        .map(|p| html_escape::encode_text(p.as_str()).to_string())
        .ok_or_else(|| MarkdownError::ParseError("Empty strikethrough content".to_string()))
}

fn process_underline_content(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    pair.into_inner()
        .next()
        .map(|p| html_escape::encode_text(p.as_str()).to_string())
        .ok_or_else(|| MarkdownError::ParseError("Empty underline content".to_string()))
}

/// Process markdown links [text](url).
fn process_link(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    let mut inner = pair.into_inner();
    let text = inner
        .next()
        .map(|p| p.into_inner().as_str())
        .ok_or_else(|| MarkdownError::ParseError("Missing link text".to_string()))?;
    let url = inner
        .next()
        .map(|p| p.as_str())
        .ok_or_else(|| MarkdownError::ParseError("Missing link URL".to_string()))?;

    Ok(format!(
        "<a href=\"{}\">{}</a>",
        url,
        html_escape::encode_text(text)
    ))
}

/// Process markdown images ![alt](url).
fn process_image(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    let mut inner = pair.into_inner();
    let alt = inner
        .next()
        .map(|p| p.into_inner().as_str())
        .ok_or_else(|| MarkdownError::ParseError("Missing image alt text".to_string()))?;
    let url = inner
        .next()
        .map(|p| p.as_str())
        .ok_or_else(|| MarkdownError::ParseError("Missing image URL".to_string()))?;

    Ok(format!(
        "<img src=\"{}\" alt=\"{}\">",
        url,
        html_escape::encode_text(alt)
    ))
}

fn process_document_quote(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    process_quote(pair)
}

fn process_quote_line(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    let inner = pair.into_inner().next();
    match inner {
        Some(content) => {
            let html = convert_to_html(content)?;
            Ok(format!("<p>{}</p>", html))
        }
        None => Ok("<p></p>".to_string()),
    }
}

fn process_quote(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    let mut lines: Vec<String> = Vec::new();

    for line in pair.into_inner() {
        let processed = process_quote_line(line)?;
        if !processed.is_empty() {
            lines.push(processed);
        }
    }

    Ok(format!("<blockquote>\n{}\n</blockquote>", lines.join("\n")))
}

fn process_code_fence(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    process_code_block(pair)
}

fn process_unordered_list(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    let items: Result<Vec<String>, MarkdownError> = pair
        .into_inner()
        .map(process_list_item)
        .collect();

    Ok(format!("<ul>\n{}\n</ul>", items?.join("\n")))
}

fn process_ordered_list(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    let items: Result<Vec<String>, MarkdownError> = pair
        .into_inner()
        .map(process_list_item)
        .collect();

    Ok(format!("<ol>\n{}\n</ol>", items?.join("\n")))
}

fn process_list_item(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    let content = pair.as_str();
    let text = content
        .find(char::is_whitespace)
        .map(|pos| &content[pos + 1..])
        .unwrap_or("")
        .trim_end_matches('\n')
        .trim();

    Ok(format!("<li>{}</li>", html_escape::encode_text(text)))
}

fn process_code_block(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    let mut inner = pair.into_inner();
    let language = inner
        .next()
        .map(|p| p.as_str().trim())
        .unwrap_or("");
    let code = inner
        .next()
        .map(|p| html_escape::encode_text(p.as_str()).to_string())
        .unwrap_or_default();

    let lang_attr = if language.is_empty() {
        String::new()
    } else {
        format!(" class=\"language-{}\"", language)
    };

    Ok(format!("<pre><code{lang_attr}>{code}</code></pre>"))
}

fn process_escape_sequence(pair: Pair<Rule>) -> Result<String, MarkdownError> {
    let escaped = pair
        .into_inner()
        .next()
        .map(|p| p.as_str())
        .unwrap_or("");
    Ok(html_escape::encode_text(escaped).to_string())
}

/// Convert markdown file to HTML file.
/// Reads markdown from input path and writes HTML to output path.
///
/// # Arguments
/// * `input_path` - Path to markdown file
/// * `output_path` - Path where HTML will be written
///
/// # Returns
/// Ok(()) on success or MarkdownError
pub fn convert_file_to_html(input_path: &Path, output_path: &Path) -> Result<(), MarkdownError> {
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);

    let mut content = String::new();
    for line in reader.lines() {
        content.push_str(&line?);
        content.push('\n');
    }

    let html_lines = str_to_html(&content)?;

    let mut output = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_path)?;

    for line in html_lines {
        writeln!(output, "{}", line)?;
    }

    Ok(())
}

/// Print HTML conversion result to console.
/// Useful for command-line usage.
///
/// # Arguments
/// * `input` - Markdown text to convert and print
///
/// # Returns
/// Ok(()) on success or MarkdownError
pub fn print_html_to_console(input: &str) -> Result<(), MarkdownError> {
    let html_lines = str_to_html(input)?;
    for line in html_lines {
        println!("{}", line);
    }
    Ok(())
}