use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use ast::LabelId;
use clap::Parser as ClapParser;

use errors::{generator_error_into_diagnostic, parse_error_into_diagnostic, TerminalEmitter};
use generator::Generator;
use lexer::{Lexer, Span};
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

pub struct LabelManager {
    map: Vec<(String, Option<i8>, Option<Span>)>,
}

impl LabelManager {
    pub fn new() -> Self {
        Self { map: Vec::new() }
    }

    pub fn get_id_of(&self, label: &str) -> Option<LabelId> {
        self.map.iter().position(|l| l.0 == label)
    }

    pub fn insert_unique(&mut self, label: &str, label_span: Span) -> Result<LabelId, ()> {
        let exists = self.map.iter().any(|l| l.0 == label);

        if exists {
            Err(())
        } else {
            self.map.push((String::from(label), None, Some(label_span)));
            Ok(self.map.len() - 1)
        }
    }

    pub fn get_or_insert_reference(&mut self, label: &str) -> LabelId {
        let exists = self.map.iter().any(|l| l.0 == label);

        if exists {
            self.get_id_of(label).unwrap()
        } else {
            self.map.push((String::from(label), None, None));
            self.map.len() - 1
        }
    }

    /// Sets the value of a label (the byte index that it refers to)
    ///
    /// Returns Err(()) when the label specified does not exist
    pub fn set_value_of(&mut self, id: LabelId, value: i8) -> Result<(), ()> {
        self.map.get_mut(id).map(|l| l.1 = Some(value)).ok_or(())
    }

    /// Sets the span of a label (the place where it is defined in the source)
    ///
    /// Returns Err(()) when the label specified does not exist
    pub fn set_span_of(&mut self, id: LabelId, span: Span) -> Result<(), ()> {
        self.map.get_mut(id).map(|l| l.2 = Some(span)).ok_or(())
    }

    pub fn get_value_of(&self, id: LabelId) -> Option<i8> {
        self.map.get(id).map(|l| l.1).flatten()
    }

    pub fn get_span_of(&self, id: LabelId) -> Option<Span> {
        self.map.get(id).map(|l| l.2).flatten()
    }
}

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

    let null_bytes = 64 - output.len();

    for _ in 0..null_bytes {
        output.push(0);
    }

    let mut col = 1;

    for b in output.iter() {
        print!("{b:02x} ");

        if col == 8 {
            println!();
            col = 0;
        }

        col += 1;
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
