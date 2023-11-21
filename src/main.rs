use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use clap::Parser as ClapParser;

use errors::{generator_error_into_diagnostic, parse_error_into_diagnostic, TerminalEmitter};
use generator::{Generator, INSTRUCTION_MEMORY_SIZE_BYTES};
use lexer::Lexer;
use parser::Parser;
use sources::SourceManager;

use crate::lexer::TokenType;

mod ast;
mod errors;
mod generator;
mod instructions;
mod lexer;
mod parser;
mod sources;

#[derive(ClapParser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(help = "Input assembly language file")]
    file: String,

    #[arg(
        long,
        short,
        help = "Output binary file path. Default is input file with .bin extension"
    )]
    output_path: Option<String>,

    #[arg(long, short, help = "Helpful for debugging the assembler itself")]
    debug: bool
}

fn main() {
    let args = Args::parse();

    let source = match File::open(&args.file) {
        Ok(mut file) => {
            let mut contents = String::new();

            if let Err(e) = file.read_to_string(&mut contents) {
                eprintln!("File read error: {e}");
                return;
            }

            contents
        }
        Err(e) => {
            eprintln!("File read error: {e}");
            return;
        }
    };

    let source_manager = SourceManager::new(&source, args.file.clone());

    let mut lexer = Lexer::new(&source);

    let tokens = lexer.lex();
    let mut valid_tokens = Vec::with_capacity(tokens.capacity());

    for token in tokens {
        if token.tt == TokenType::InvalidTokenError {
            let text = source_manager.get_span(token.span).unwrap();
            eprintln!("Invalid token found `{}`", text);
        } else if token.tt == TokenType::InvalidIntegerError {
            let text = source_manager.get_span(token.span).unwrap();
            eprintln!("Invalid integer value `{}`", text);
        } else if token.tt != TokenType::Comment {
            valid_tokens.push(token);
        }
    }

    let parser = Parser::new(&valid_tokens, &source_manager);

    let (items, label_manager) = match parser.parse() {
        Ok((items, label_manager)) => (items, label_manager),
        Err(e) => {
            TerminalEmitter::emit(
                parse_error_into_diagnostic(e, &source_manager),
                &source_manager,
            );
            return;
        }
    };

    let mut generator = Generator::new(items, label_manager);
    let mut output = match generator.generate() {
        Ok(output) => output,
        Err(e) => {
            TerminalEmitter::emit(
                generator_error_into_diagnostic(e, &source_manager),
                &source_manager,
            );
            return;
        }
    };

    let num_null_bytes = INSTRUCTION_MEMORY_SIZE_BYTES as usize - output.len();

    for _ in 0..num_null_bytes {
        output.push(0);
    }

    if args.debug {
        debug_print_output(&output);
    }

    let output_path = args.output_path.unwrap_or_else(|| {
        let mut output_file = PathBuf::from(args.file);
        output_file.set_extension("bin");
        String::from(output_file.to_str().unwrap())
    });

    match File::create(output_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(&output) {
                eprintln!("File write error: {e}");
            }
        }
        Err(e) => {
            eprintln!("File write error: {e}");
        }
    }
}

fn debug_print_output(output: &Vec<u8>) {
    let mut col = 1;

    for b in output.iter() {
        print!("{b:02x} ");

        if col == 8 {
            println!();
            col = 0;
        }

        col += 1;
    }
}