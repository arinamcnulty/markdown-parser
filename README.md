# Markdown Parser

A comprehensive Markdown to HTML converter built in Rust using Pest grammar. This project provides a command-line interface and library for parsing Markdown documents.

## Installation

### From crates.io

The package is published on [crates.io](https://crates.io/crates/arinamcnulty-markdown-parser).

```bash
cargo install arinamcnulty-markdown-parser
```

### From Source

```bash
git clone https://github.com/arinamcnulty/markdown-parser.git
cd markdown-parser
cargo build --release
```

## Usage

You can read documentation on [docs.rs](https://docs.rs/arinamcnulty-markdown-parser)

### Command Line Interface

#### Convert a Markdown file to HTML

```bash
markdown_parser convert -i document.md -o document.html
```

#### Parse Markdown text directly

```bash
markdown_parser parse -t "# Hello World!"
```

#### Display help and credits

```bash
markdown_parser info
```

### Library Usage

```rust
use markdown_parser::{str_to_html, convert_file_to_html};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse markdown string
    let markdown = "# Hello World\n\nThis is **bold** text with `inline code`.";
    let html = str_to_html(markdown)?;
    println!("{}", html.join("\n"));

    // Convert file
    convert_file_to_html("input.md", "output.html")?;

    Ok(())
}
```

## Grammar Examples

This parser supports the full CommonMark Markdown specification. Here are examples of supported syntax:

### Headings

```markdown
# Heading Level 1
## Heading Level 2
### Heading Level 3
```

### Text Formatting

```markdown
**Bold text**
*Italic text*
~~Strikethrough text~~
__Underline text__
```

### Links and Images

```markdown
[Click here](https://example.com)
![Alt text](image.png)
```

### Inline Code

```markdown
Use `code` for inline code snippets.
```

### Code Blocks

````markdown
```rust
fn main() {
    println!("Hello, world!");
}
```
````

### Lists

#### Unordered Lists
```markdown
- Item 1
- Item 2
* Another item
```

#### Ordered Lists
```markdown
1. First item
2. Second item
3. Third item
```

### Blockquotes

```markdown
> This is a blockquote
> Second line of blockquote
```

### Horizontal Rules

```markdown
---
***
___
```

### Escaped Characters

```markdown
\*literal asterisk\*
\[literal bracket\]
```

## Grammar Structure

The parser uses Pest grammar for efficient parsing. The grammar is organized into the following main components:

### Document Structure
```
document_structure = { SOI ~ (document_block ~ NEWLINE*)* ~ document_block? ~ EOI? }
document_block = {
    document_heading
  | document_quote
  | code_fence
  | document_unordered_list
  | document_ordered_list
  | thematic_break
  | document_paragraph
}
```

### Inline Content
```
inline_content = _{
    image
  | link
  | text_formatting
  | inline_code
  | escape_sequence
  | plain_text
}
```

### Text Formatting
```
text_formatting = _{
    bold_formatting
  | italic_formatting
  | strikethrough_formatting
  | underline_formatting
}
```

## Testing

Run the test suite:

```bash
cargo test
```

Run with formatting and linting:

```bash
cargo fmt
cargo clippy
```

## API Documentation

### Core Functions

- `parse_markdown(input: &str)` - Parse markdown string to syntax tree
- `str_to_html(input: &str)` - Convert markdown string to HTML vector
- `convert_file_to_html(input: &Path, output: &Path)` - Convert markdown file to HTML file
- `print_html_to_console(input: &str)` - Print HTML conversion to stdout

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum MarkdownError {
    #[error("Parsing failed: {0}")]
    ParseError(String),

    #[error("File operation failed: {0}")]
    IoError(#[from] std::io::Error),
}
```

## Development

### Project Structure

```
src/
├── main.rs          # CLI application
├── lib.rs           # Library implementation
└── grammar.pest     # Pest grammar rules

tests/
└── grammar_tests.rs # Unit tests
```

### Adding New Grammar Rules

1. Add rule to `grammar.pest`
2. Implement conversion function in `lib.rs`
3. Add rule to `convert_to_html` match statement
4. Add unit tests in `tests/grammar_tests.rs`

### Building for Development

```bash
cargo build
cargo run -- info
```

### Code Quality

- **Formatting**: `cargo fmt`
- **Linting**: `cargo clippy`
- **Testing**: `cargo test`


## Author

**Zudilova Oryna** - *Initial work* - [arinamcnulty](https://github.com/arinamcnulty)
