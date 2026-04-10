use lace_frontend::{lexer, parser, semantics_checker, ir_gen, utils};
use clap::Parser;
use tinycolor::Colorize;
use lasso::Rodeo;
use std::fs;
use std::str::FromStr;

#[derive(Clone)]
pub enum Target {
    Bytecode,
    JavaScript,
    C,
}

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bytecode" | "byte-code" | "Bytecode" | "ByteCode" => Ok(Self::Bytecode),
            "js" | "javascript" | "java-script" | "JS" | "JavaScript" => Ok(Self::JavaScript),
            "c" | "C" => Ok(Self::C),
            s => Err(format!("invalid target: `{s}`"))
        }
    }
}

#[derive(Parser)]
#[command(
    bin_name = "lacec",
    about = "The Lace Language Compiler",
    version,
)]
struct Cli {
    #[arg(help = "The input file to compile")]
    input: String,
    
    #[arg(
        short,
        long,
        help = "The target (bytecode/JavaScript/C)"
    )]
    target: Option<Target>,

    #[arg(
        short,
        long,
        help = "For debugging phases of compilation (tokens -> parse tree -> IR -> bytecode or JavaScript/C code)"
    )]
    emit: Option<Vec<String>>,

    #[arg(
        short, long,
        value_parser = clap::value_parser!(u32).range(0..=3),
        help = "The optimization level (from 0 to 3)"
    )]
    opt_level: Option<u32>
}

fn main() {
    let cli = Cli::parse();

    let source = match fs::read_to_string(&cli.input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: {e}", "error".red().bold());
            return;
        }
    };

    let line_starts = utils::line_starts(&source);
    let lines = source.lines().collect::<Vec<_>>();

    let mut emit_tokens = false;
    let mut emit_ast = false;
    let mut emit_ir = false;
    if let Some(emits) = &cli.emit {
        for emit in emits {
            if ["tokens", "toks"].contains(&&**emit) {
                emit_tokens = true;
            } else if ["ast", "parse_tree", "parse-tree"].contains(&&**emit) {
                emit_ast = true;
            } else if ["ir", "intermediate-representation"].contains(&&**emit) {
                emit_ir = true;
            }
        }
    }

    let mut rodeo = Rodeo::new();

    let tokens = match lexer::tokenize(&source, &mut rodeo) {
        Ok(toks) => toks,
        Err(e) => {
            eprintln!("{}: {}", "error".red().bold(), e.display(&cli.input, &lines, &line_starts));
            eprintln!("cannot continue compilation due to {} previous error(s)", "1".red().bold());
            return;
        },
    };

    if emit_tokens {
        println!("TOKENS: [");
        for (idx, tok) in tokens.tokens.iter().enumerate() {
            if idx > 0 {
                println!(",");
            }
            print!("\t`{}`", tok.kind.as_str(&rodeo));
        }
        println!("\n]");
    }

    let mut parser = parser::Parser::new(tokens, &rodeo);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("{}: {}", "error".red().bold(), e.display(&cli.input, &lines, &line_starts));
            eprintln!("cannot continue compilation due to {} previous error(s)", "1".red().bold());
            return;
        },
    };

    if emit_ast {
        println!("AST: {:#?}", ast.0);
    }

    let mut schecker = semantics_checker::SemanticsChecker::new(&mut rodeo);
    match schecker.check(&ast) {
        Ok(_) => (),
        Err(e) => {
            for err in &e {
                eprintln!("{}: {}", "error".red().bold(), err.display(&cli.input, &lines, &line_starts));
            }
            eprintln!("cannot continue compilation due to {} previous error(s)", e.len().to_string().red().bold());
            return;
        },
    };

    let mut ir_gen = ir_gen::IRGenerator::new(0, &cli.input);
    ir_gen.generate_ir(&ast, &schecker.type_map);
    let ir_mod = ir_gen.module;
    if emit_ir {
        println!("{}", ir_mod.debug(&rodeo));
    }
}