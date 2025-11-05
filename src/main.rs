use arinamcnulty_markdown_parser::{MarkdownError, convert_file_to_html, print_html_to_console};
use clap::{Arg, ArgMatches, Command};
use std::path::PathBuf;

fn main() -> Result<(), MarkdownError> {
    let app = build_cli_app();
    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("convert", sub_matches)) => handle_convert_command(sub_matches),
        Some(("parse", sub_matches)) => handle_parse_command(sub_matches),
        Some(("info", _)) => handle_info_command(),
        _ => {
            println!("No valid command provided. Use --help for usage information.");
            Ok(())
        }
    }
}

fn build_cli_app() -> Command {
    Command::new("markdown-parser")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Zudilova Oryna <zarinamcnulty@gmail.com>")
        .about("Advanced Markdown to HTML converter with CLI support")
        .long_about(
            "A command-line tool for converting Markdown documents to HTML.\n\
             Supports various Markdown features including headings, formatting, links, images and more."
        )
        .subcommand(
            Command::new("convert")
                .about("Convert Markdown file to HTML file")
                .long_about(
                    "Reads a Markdown file from disk and converts it to HTML format.\n\
                     The output file will be created or overwritten if it already exists."
                )
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("INPUT_FILE")
                        .help("Path to the input Markdown file")
                        .required(true)
                        .value_parser(clap::value_parser!(PathBuf))
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("OUTPUT_FILE")
                        .help("Path where the HTML output will be written")
                        .required(true)
                        .value_parser(clap::value_parser!(PathBuf))
                )
        )
        .subcommand(
            Command::new("parse")
                .about("Parse Markdown text from command line")
                .long_about(
                    "Accepts Markdown text directly from the command line or reads from\n\
                     a file and outputs the corresponding HTML. Useful for quick testing\n\
                     and processing markdown files."
                )
                .arg(
                    Arg::new("text")
                        .short('t')
                        .long("text")
                        .value_name("MARKDOWN_TEXT")
                        .help("Markdown text to parse and convert")
                        .required(true)
                        .conflicts_with("input")
                )
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("INPUT_FILE")
                        .help("Path to Markdown file to parse and convert")
                        .value_parser(clap::value_parser!(PathBuf))
                )
        )
        .subcommand(
            Command::new("info")
                .about("Display application and library information")
                .long_about(
                    "Shows detailed information about the application, including\n\
                     version, supported features, and usage examples."
                )
        )
        .arg_required_else_help(true)
}

fn handle_convert_command(matches: &ArgMatches) -> Result<(), MarkdownError> {
    let input_path: &PathBuf = matches.get_one("input").expect("Input path is required");
    let output_path: &PathBuf = matches.get_one("output").expect("Output path is required");

    println!(
        "🔄 Converting '{}' to '{}'...",
        input_path.display(),
        output_path.display()
    );

    match convert_file_to_html(input_path, output_path) {
        Ok(()) => {
            println!("✅ Conversion completed successfully!");
            println!("📄 HTML file saved to: {}", output_path.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Conversion failed: {}", e);
            Err(e)
        }
    }
}

fn handle_parse_command(matches: &ArgMatches) -> Result<(), MarkdownError> {
    let markdown_text = if let Some(input_path) = matches.get_one::<PathBuf>("input") {
        std::fs::read_to_string(input_path)?
    } else {
        matches
            .get_one::<String>("text")
            .expect("Either text or input file is required")
            .clone()
    };

    print_html_to_console(&markdown_text)
}

fn handle_info_command() -> Result<(), MarkdownError> {
    println!("Markdown Parser v{}", env!("CARGO_PKG_VERSION"));
    println!("Advanced Markdown to HTML converter");
    println!();
    println!("Features:");
    println!("  • Complete Markdown syntax support");
    println!("  • HTML output with proper escaping");
    println!("  • File and text processing modes");
    println!();
    println!("Usage Examples:");
    println!();
    println!("Convert a file:");
    println!("  markdown-parser convert -i document.md -o document.html");
    println!();
    println!("Parse text directly:");
    println!("  markdown-parser parse -t \"# Hello **World**\"");
    println!();
    println!("Show this information:");
    println!("  markdown-parser info");
    println!();
    println!("For issues and contributions:");
    println!("  https://github.com/arinamcnulty/markdown-parser");

    Ok(())
}
