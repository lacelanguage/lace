//! # The Lace Language Frontend
//! 
//! ## Core Structure
//! 
//! - **Lexer:** Handles lexing (turning source code into parseable tokens)
//! - **Parser:** Takes tokens and creating an Abstract Syntax Tree (ensures syntactical correctness)
//! - **Semantics Checker:** Handles checking if a program makes sense and outputs a type map for each node in the AST (ensures semantic correctness)
//! - **IR Generator:** Turns the AST into a flat Intermediate Representation
//! 
//! ## Pipeline
//! 
//! ```plaintext
//! [ Source Code ] → { Lexer }
//!                       ↓
//!   { Parser }  ←  [ Tokens ]
//!       ↓
//!    [ AST ]   →   { Semantics Checker }
//!                            ↓
//! { IR Generator } ← [ AST + Type Map ]
//!        ↓
//!   [ Lace IR ]
//! ```
//! 
//! ## Utilities
//! 
//! - [`span::Span`] for representing windows into the source code
//! - [`operator::Op`] for representing operators in a central way
//! - [`diagnostic::Diagnostic`] for errors, warnings, notes, hints, etc.
//! - Utility functions (e.g. [`utils::create_snippet`], [`utils::line_starts`], [`utils::line_of`], etc.) in `src/utils.rs`

pub mod lexer;
pub mod parser;
pub mod semantics_checker;
pub mod ir_gen;

pub mod operator;
pub mod diagnostic;
pub mod utils;