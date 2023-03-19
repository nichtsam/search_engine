use clap::{CommandFactory, Parser, Subcommand};
use search_engine::{index_dir, read_index, save_index, search, DocumentTermsFrequenciesIndex};
use std::time::SystemTime;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "index the specified path recursively")]
    Index {
        #[arg(help = "the directory of the collections you want to index.")]
        input_dir: String,
        #[arg(help = "the path to output the index result for further usage.")]
        output_path: String,
    },
    #[command(about = "lists out top 10 most relevant document")]
    Search {
        #[arg(help = "the word or phrase that you’d like to rank for.")]
        keyword_phrase: String,
        #[arg(help = "the path of the document terms frequencies index to search the term in.")]
        dtf_index_path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let start = SystemTime::now();

    match &cli.command {
        Commands::Index {
            input_dir,
            output_path,
        } => {
            let mut dtf_index = DocumentTermsFrequenciesIndex::new();

            if let Err(err) = index_dir(input_dir, &mut dtf_index) {
                Cli::command().error(clap::error::ErrorKind::Io, err).exit();
            };

            if let Err(err) = save_index(&dtf_index, output_path) {
                eprintln!("ERROR: could not save index to path {output_path}: {err}");
            }
        }

        Commands::Search {
            keyword_phrase,
            dtf_index_path,
        } => {
            let dtf_index: DocumentTermsFrequenciesIndex = read_index(dtf_index_path)
                .unwrap_or_else(|err| {
                    Cli::command().error(clap::error::ErrorKind::Io, err).exit();
                });

            search(keyword_phrase, &dtf_index);
        }
    }

    let end = SystemTime::now();
    let duration = end.duration_since(start).unwrap().as_secs_f32();
    println!("operation took {} seconds", duration);
}
