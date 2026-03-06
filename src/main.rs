use clap::{Args, Parser, Subcommand, ValueEnum};
use osta_ast::debug::AstPrinter;
use osta_lexer::Lexer;
use osta_parser::parse;
use osta_session::Session;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    NoBuild(NoBuildArgs),
}

#[derive(Args, Debug)]
struct NoBuildArgs {
    input_file: PathBuf,

    #[clap(short, long)]
    #[arg(value_enum, value_delimiter = ',', default_value = "tokens,ast/dot")]
    artifacts: Vec<NoBuildArtifact>,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, ValueEnum)]
enum NoBuildArtifact {
    #[clap(name = "tokens")]
    TokenStream,
    #[clap(name = "ast/dot")]
    AstDot,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Command::NoBuild(NoBuildArgs { input_file, artifacts }) => {
            let source = std::fs::read_to_string(&input_file).unwrap();
            for artifact in artifacts {
                match artifact {
                    NoBuildArtifact::TokenStream => gen_token_stream(&source),
                    NoBuildArtifact::AstDot => gen_ast_dot(&source),
                }
            }
        }
    }
}

fn gen_token_stream(source: &str) {
    let mut string_builder = String::new();
    let mut valid = true;
    let lexer = Lexer::new(source);
    for entry in lexer {
        match entry {
            Ok(token) => {
                string_builder = format!("{string_builder}{token:?}\n");
            }
            Err(e) => {
                eprintln!("Skipping generation of TokenStream:\n{e:?}");
                valid = false;
                break;
            }
        }
    }
    if valid {
        println!("{string_builder}");
    }
}

fn gen_ast_dot(source: &str) {
    let mut session = Session::create();
    let ast = parse(&mut session, source).unwrap();

    let printer = AstPrinter::new(&session, source, &ast);
    println!("{printer}");
}
