#[macro_use]
extern crate log;
use data::{HistoricalData, Token};
use pretty_env_logger;

mod data;
mod request;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();

    let token_1 = Token::Bitcoin;
    let token_2 = Token::Snp500;
    let coviarence = HistoricalData::calculate_covariance(token_1, token_2).await?;

    println!("{:#?}", coviarence);
    Ok(())
}
