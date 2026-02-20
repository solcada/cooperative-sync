use rust_client::{SyncClient, SyncRequest};

fn main() {
    if let Err(message) = run() {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 4 {
        print_usage();
        return Err("missing required arguments".to_string());
    }

    let command = args[1].as_str();
    let server = if args[2] == "--server" {
        args[3].as_str()
    } else {
        return Err("expected --server <URL>".to_string());
    };

    let client = SyncClient::new(server).map_err(|err| err.to_string())?;

    match command {
        "health" => {
            let ok = client.health_check().map_err(|err| err.to_string())?;
            if ok {
                println!("server is healthy");
            } else {
                println!("server is reachable but unhealthy");
            }
        }
        "sync" => {
            if args.len() != 8 {
                print_usage();
                return Err("sync requires --path and --hash".to_string());
            }

            if args[4] != "--path" || args[6] != "--hash" {
                return Err("expected sync --server <URL> --path <PATH> --hash <HASH>".to_string());
            }

            let request = SyncRequest {
                path: args[5].clone(),
                hash: args[7].clone(),
            };

            client.sync_file(&request).map_err(|err| err.to_string())?;
            println!("sync request queued");
        }
        _ => {
            print_usage();
            return Err(format!("unknown command: {command}"));
        }
    }

    Ok(())
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  rust-client health --server <http://host:port>");
    eprintln!("  rust-client sync --server <http://host:port> --path <relative/path> --hash <sha256>");
}
