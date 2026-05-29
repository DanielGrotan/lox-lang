use std::{
    env, fs,
    io::{self, BufRead, Write},
    process,
};

mod error;
mod lexer;
mod parser;

use crate::{
    error::{Error, Result},
    lexer::{Lexer, TokenKind},
};

fn main() -> Result<()> {
    let mut args = env::args();

    let program_path = args.next().expect("missing program path");

    if args.len() > 1 {
        println!("Usage: {program_path} [script]");
        process::exit(0);
    }

    if let Some(script_path) = args.next() {
        run_script(script_path)
    } else {
        run_repl()
    }
}

fn run_script(script_path: String) -> Result<()> {
    let content = fs::read(script_path).map_err(|_| Error::ScriptNotFound)?;
    let src = String::from_utf8(content).map_err(|_| Error::InvalidEncoding)?;

    run(&src)
}

fn run_repl() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut handle = stdin.lock();
    let mut line = String::new();

    loop {
        print!("> ");
        let _ = stdout.flush();

        line.clear();

        let bytes = handle.read_line(&mut line).expect("invalid encoding");

        if bytes == 0 {
            break;
        }

        let input = line.trim_end();

        if input.is_empty() {
            continue;
        }

        let _ = run(input);
    }

    Ok(())
}

fn run(src: &str) -> Result<()> {
    let mut lexer = Lexer::new(src);

    loop {
        let token = lexer.next_token();

        match token.kind {
            TokenKind::Eof => break,
            _ => println!("{token:?}"),
        }
    }

    Ok(())
}
