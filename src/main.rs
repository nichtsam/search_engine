use clap::{CommandFactory, Parser, Subcommand};
use search_engine::{
    io::{read_model, write_model},
    Model,
};
use std::time::SystemTime;
use tiny_http::{Response, Server, StatusCode};

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
        #[arg(help = "the path to the model(indexed collections output) to search the term in.")]
        model_path: String,
    },
    #[command(about = "spin up a search engine server")]
    Serve {
        #[arg(help = "the path to the model(indexed collections output) to search the term in.")]
        model_path: String,
        #[arg(short, long, default_value_t = 42069)]
        port: u16,
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
            let mut model = Model::default();

            if let Err(err) = model.add_documents(input_dir) {
                Cli::command().error(clap::error::ErrorKind::Io, err).exit();
            };

            if let Err(err) = write_model(&model, output_path) {
                eprintln!("ERROR: could not save index to path {output_path}: {err}");
            }
        }

        Commands::Search {
            keyword_phrase,
            model_path,
        } => {
            let model = read_model(model_path).unwrap_or_else(|err| {
                Cli::command().error(clap::error::ErrorKind::Io, err).exit();
            });

            model.search(keyword_phrase)
        }

        Commands::Serve { model_path, port } => {
            let model = read_model(model_path).unwrap_or_else(|err| {
                eprintln!("ERROR: could not read model from {model_path}: {err}");
                std::process::exit(1)
            });

            let addr = format!("127.0.0.1:{port}");
            println!("serving at {addr}");

            let server = Server::http(&addr).unwrap_or_else(|err| {
                eprintln!("ERROR: could not start server on {addr}: {err}");
                std::process::exit(1)
            });

            for request in server.incoming_requests() {
                println!(
                    "received request! method: {:#?}, url: {:#?}",
                    request.method(),
                    request.url(),
                );

                match request.url() {
                    "/api/search" => {
                        let response = Response::from_string("hello world");

                        request.respond(response).unwrap_or_else(|err| {
                            eprintln!("ERROR: could not repond to request: {err}");
                        });
                    }
                    _ => {
                        let response =
                            Response::from_string("404").with_status_code(StatusCode(404));

                        request.respond(response).unwrap_or_else(|err| {
                            eprintln!("ERROR: could not repond to request: {err}");
                        });
                    }
                }
            }
        }
    }

    let end = SystemTime::now();
    let duration = end.duration_since(start).unwrap().as_secs_f32();
    println!("operation took {} seconds", duration);
}
