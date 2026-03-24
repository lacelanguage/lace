use lace_frontend::lexer;
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

    let mut emit_tokens = false;
    if let Some(emits) = &cli.emit {
        for emit in emits {
            if ["tokens", "toks", "t"].contains(&&**emit) {
                emit_tokens = true;
            }
        }
    }

    let mut rodeo = Rodeo::new();

    let tokens = match lexer::tokenize(&source, &mut rodeo) {
        Ok(toks) => toks,
        Err(e) => {
            eprintln!("{}: {e:?}", "error".red().bold());
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
}