mod angular;
mod client;

use angular::convert_angular_docs;
use client::AppState;
use std::env;

fn usage() -> ! {
    eprintln!("Usage: angular-docs-to-md <URL> [options]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --examples N    tab examples to expand per group (default: 1)");
    eprintln!("  --no-header     skip <docs-decorative-header> parsing");
    eprintln!("  --no-pills      skip <docs-pill-row> parsing");
    eprintln!();
    eprintln!("Example:");
    eprintln!("  angular-docs-to-md https://angular.dev/guide/components --examples 2");
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        usage();
    }

    let url = &args[0];

    if !url.contains("angular.dev") {
        eprintln!("error: URL must be from angular.dev");
        std::process::exit(1);
    }

    let examples: usize = args.windows(2)
        .find(|w| w[0] == "--examples")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(1);

    let parse_header = !args.iter().any(|a| a == "--no-header");
    let parse_pills  = !args.iter().any(|a| a == "--no-pills");

    let client = &AppState::new().client;

    match convert_angular_docs(client, url, examples, parse_header, parse_pills).await {
        Ok(md)  => print!("{md}"),
        Err(e)  => { eprintln!("error: {e}"); std::process::exit(1); }
    }
}
