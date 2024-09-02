use crate::data::{HistoricalData, Token};
use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CovarianceQuery {
    token_1: Option<String>,
    token_2: Option<String>,
}

#[derive(Deserialize)]
pub struct VolatilityQuery {
    token: Option<String>,
}

#[get("/covariance")]
pub async fn get_covariance(query: web::Query<CovarianceQuery>) -> impl Responder {
    // Extract token_1 and token_2 strings from the query parameters
    let token_1_str = match &query.token_1 {
        Some(token) => token,
        None => return HttpResponse::BadRequest().body("Missing query parameter: token_1"),
    };

    let token_2_str = match &query.token_2 {
        Some(token) => token,
        None => return HttpResponse::BadRequest().body("Missing query parameter: token_2"),
    };

    // Convert the token strings to the enum values
    let token_1 = match Token::from_str(token_1_str) {
        Some(token) => token,
        None => {
            return HttpResponse::BadRequest()
                .body(format!("Invalid token_1 value: {}", token_1_str))
        }
    };

    let token_2 = match Token::from_str(token_2_str) {
        Some(token) => token,
        None => {
            return HttpResponse::BadRequest()
                .body(format!("Invalid token_2 value: {}", token_2_str))
        }
    };

    match HistoricalData::calculate_covariance(token_1, token_2).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/volatility")]
pub async fn get_volatility(query: web::Query<VolatilityQuery>) -> impl Responder {
    let token_str = match &query.token {
        Some(token) => token,
        None => return HttpResponse::BadRequest().body("Missing query parameter: token"),
    };

    let token = match Token::from_str(token_str) {
        Some(token) => token,
        None => {
            return HttpResponse::BadRequest().body(format!("Invalid token value: {}", token_str))
        }
    };

    match HistoricalData::calculate_realized_volatility(token).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
