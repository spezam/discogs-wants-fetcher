use clap::Parser;
use discogs_wants_fetcher::client::DiscogsClient;
use owo_colors::OwoColorize;

const BANNER: &str = include_str!("../banner");

#[derive(Parser, Debug)]
#[command(version, after_help = "eof")]
struct CliArgs {
    #[clap(short, long, help = "Discogs username")]
    username: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    println!("{}", BANNER.to_string().red());

    let args = CliArgs::parse();
    let client = DiscogsClient::new();

    match client.get_wants_raw(&args.username).await {
        Ok(response) => {
            println!("{:?}", response);
        }
        Err(err) => {
            println!("{err:?}");
        }
    }

    Ok(())
}
