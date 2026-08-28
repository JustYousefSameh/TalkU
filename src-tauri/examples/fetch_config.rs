use std::path::PathBuf;

use talku_lib::config;

fn main() {
    let mut args = std::env::args().skip(1);
    let api_key = match args.next() {
        Some(k) => k,
        None => {
            eprintln!("usage: fetch_config <apiKey> [configPath]");
            std::process::exit(2);
        }
    };
    let path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("talkuwg.conf"));

    let runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    match runtime.block_on(config::load_or_fetch_config(&path, &api_key)) {
        Ok(config) => {
            println!("==== ServerConfig ====");
            println!("{:#?}", config.server);
            println!("==== PrivateKey ====");
            println!("{}", config.private_key);
            println!("==== Config file ====");
            println!("{}", path.display());
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
