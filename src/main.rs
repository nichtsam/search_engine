use clap::{CommandFactory, Parser, Subcommand};
use search_engine::{
    io::{read_model, write_model},
    Model,
};
use std::time::SystemTime;
use tiny_http::Server;

use crate::server::serve_request;

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

            let result = model.search(keyword_phrase);

            for (index, (path, rank_score)) in result.iter().enumerate().take(10) {
                println!(
                    "{no}. {path} => {rank_score}",
                    no = index + 1,
                    path = path.display()
                );
            }
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
                serve_request(request, &model).unwrap_or_else(|err| {
                    eprintln!("ERROR: could not serve request: {err}");
                });
            }
        }
    }

    let end = SystemTime::now();
    let duration = end.duration_since(start).unwrap().as_secs_f32();
    println!("operation took {} seconds", duration);
}

mod server {
    use std::{collections::HashMap, io};

    use search_engine::Model;
    use tiny_http::{Header, Method, Request, Response, StatusCode};

    pub fn serve_request(request: Request, model: &Model) -> io::Result<()> {
        let (path, query) = parse_url(request.url());
        match (request.method(), path) {
            (Method::Get, "/api/search") => {
                let keyword_phrase = query.and_then(|q| q.get("q").cloned()).unwrap_or_default();
                let search_result = model.search(keyword_phrase);
                let result = search_result.iter().take(20).collect::<Vec<_>>();

                let header = Header::from_bytes(&b"Content-type"[..], &b"application/json"[..])
                    .expect("header should be right");
                let response =
                    Response::from_string(serde_json::to_string(&result)?).with_header(header);

                request.respond(response)?;

                Ok(())
            }
            _ => serve_404(request),
        }
    }

    type Query<'a> = HashMap<&'a str, &'a str>;
    fn parse_url(url: &str) -> (&str, Option<Query>) {
        match url.split_once('?') {
            Some((path, query)) => {
                if query.is_empty() {
                    (path, None)
                } else {
                    let query = parse_query(query);
                    (path, Some(query))
                }
            }
            None => (url, None),
        }
    }

    fn parse_query(query: &str) -> Query {
        query.split('&').filter_map(|q| q.split_once('=')).collect()
    }

    fn serve_404(request: Request) -> io::Result<()> {
        let response = Response::from_string("404").with_status_code(StatusCode(404));

        request.respond(response)
    }
}
