use clap::Parser;
use discogs_wants_fetcher::client::DiscogsClient;
use owo_colors::OwoColorize;

const BANNER: &str = include_str!("../banner");

#[derive(Parser, Debug)]
#[command(version)]
struct CliArgs {
    #[clap(short, long, help = "Discogs username")]
    username: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", BANNER.red());

    let args = CliArgs::parse();
    let client = DiscogsClient::new();

    match client.get_wants_raw(&args.username).await {
        Ok(wants) => {
            for want in &wants {
                println!("{want}");
            }
        }
        Err(err) => {
            eprintln!("Error: {err}");
        }
    }

    Ok(())
}
