use clap::{Args, Parser, Subcommand, ValueEnum};
use miette::NamedSource;
use osta_ast::debug::AstPrinter;
use osta_lexer::Lexer;
use osta_parser::{parse, FileSession};
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
                let result = match artifact {
                    NoBuildArtifact::TokenStream => gen_token_stream(&source),
                    NoBuildArtifact::AstDot => gen_ast_dot(&source),
                };

                if let Err(e) = result {
                    eprintln!(
                        "{:?}",
                        e.with_source_code(NamedSource::new(
                            input_file.to_string_lossy(),
                            source.clone()
                        ))
                    );
                }
            }
        }
    }
}

fn gen_token_stream(source: &str) -> miette::Result<()> {
    let mut string_builder = String::new();
    let lexer = Lexer::new(source);
    for entry in lexer {
        match entry {
            Ok(token) => {
                string_builder = format!("{string_builder}{token:?}\n");
            }
            Err(e) => return Err(e.into()),
        }
    }
    println!("{string_builder}");
    Ok(())
}

fn gen_ast_dot(source: &str) -> miette::Result<()> {
    let FileSession { builder, interner, .. } = parse(source)?;
    let ast = builder.build();

    let printer = AstPrinter::new(&interner, source, &ast);
    println!("{printer}");

    Ok(())
}
