use std::{
    env, fs,
    io::{self, BufRead, Write},
    process,
};

mod error;
mod interpreter;
mod lexer;
mod parser;

use crate::{
    error::{CliError, Result},
    interpreter::Interpreter,
    lexer::{Lexer, TokenKind},
    parser::Parser,
};

fn main() -> Result<()> {
    let mut args = env::args();

    let program_path = args.next().expect("missing program path");

    if args.len() > 1 {
        println!("Usage: {program_path} [script]");
        process::exit(0);
    }

    let interpreter = Interpreter::new();

    if let Some(script_path) = args.next() {
        run_script(script_path, interpreter)
    } else {
        run_repl(interpreter)
    }
}

fn run_script(script_path: String, mut interpreter: Interpreter) -> Result<()> {
    let content = fs::read(script_path).map_err(|_| CliError::ScriptNotFound)?;
    let src = String::from_utf8(content).map_err(|_| CliError::InvalidEncoding)?;

    run(&src, &mut interpreter);

    Ok(())
}

fn run_repl(mut interpreter: Interpreter) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut handle = stdin.lock();
    let mut line = String::new();

    loop {
        print!("> ");
        let _ = stdout.flush();

        line.clear();

        let bytes = handle
            .read_line(&mut line)
            .map_err(|_| CliError::InvalidEncoding)?;

        if bytes == 0 {
            break;
        }

        let input = line.trim_end();

        if input.is_empty() {
            continue;
        }

        run(input, &mut interpreter);
    }

    Ok(())
}

fn run(src: &str, interpreter: &mut Interpreter) {
    let mut lexer = Lexer::new(src);
    let mut tokens = Vec::new();

    loop {
        let token = match lexer.next_token() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error: {e}");
                return;
            }
        };
        let is_eof = matches!(token.kind, TokenKind::Eof);

        tokens.push(token);

        if is_eof {
            break;
        }
    }

    let mut parser = Parser::new(tokens);
    let (program, syntax_errors) = parser.parse();

    for e in syntax_errors {
        eprintln!("{e}");
    }

    if let Err(e) = interpreter.interpret(&program) {
        eprintln!("{e}");
    };
}
