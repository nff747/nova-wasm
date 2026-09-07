//! Nova: Statically typed systems language compiling directly to WebAssembly binary format.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod typechecker;
pub mod wasm;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::typechecker::TypeChecker;
use crate::wasm::{WasmBinaryEmitter, WatEmitter};

/// Compiles Nova source code directly into valid WebAssembly binary bytecode (.wasm).
pub fn compile_to_wasm(source: &str) -> Result<Vec<u8>, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;

    let mut typechecker = TypeChecker::new();
    typechecker.check_program(&program)?;

    let mut emitter = WasmBinaryEmitter::new();
    emitter.emit(&program)
}

/// Compiles Nova source code into WebAssembly Text format (.wat).
pub fn compile_to_wat(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;

    let mut typechecker = TypeChecker::new();
    typechecker.check_program(&program)?;

    Ok(WatEmitter::emit(&program))
}
