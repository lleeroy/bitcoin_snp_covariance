#[macro_use]
extern crate log;
use std::env;

use actix_web::{middleware::Logger, App, HttpServer};
use pretty_env_logger;

mod data;
mod request;
mod server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    pretty_env_logger::init();

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .service(server::get_covariance)
            .service(server::get_volatility)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
