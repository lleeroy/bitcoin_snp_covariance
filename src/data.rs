use std::collections::{HashMap, HashSet};

use crate::request::Request;
use anyhow::anyhow;
use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
use reqwest::{
    header::{self, HeaderMap},
    Method,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct HistoricalData;

pub enum Token {
    Bitcoin,
    Snp500,
}

impl Token {
    pub fn id(&self) -> &str {
        match *self {
            Token::Bitcoin => "BTC-USD",
            Token::Snp500 => "%5EGSPC",
        }
    }

    pub fn as_string(&self) -> &str {
        match *self {
            Token::Bitcoin => "Bitcoin",
            Token::Snp500 => "SNP500",
        }
    }
}

impl HistoricalData {
    pub async fn calculate_covariance(
        token_1: Token,
        token_2: Token,
    ) -> Result<f64, anyhow::Error> {
        let token_1_data: HashMap<NaiveDate, f64> =
            Self::get_yearly_data_by_token(&token_1).await?;
        let token_2_data: HashMap<NaiveDate, f64> =
            Self::get_yearly_data_by_token(&token_2).await?;

        println!("{:#?}", token_1_data);
        println!("{:#?}", token_2_data);

        todo!("Filter out results which are not present in one of two tokens.");

        let common_dates: Vec<NaiveDate> = token_1_data
            .keys()
            .filter(|&&date| token_2_data.contains_key(&date))
            .copied()
            .collect();

        if common_dates.is_empty() {
            return Err(anyhow!(
                "No common timestamps found between the two tokens."
            ));
        }

        let mean1 =
            token_1_data.iter().map(|(_, value)| value).sum::<f64>() / token_1_data.len() as f64;

        let mean2 =
            token_2_data.iter().map(|(_, value)| value).sum::<f64>() / token_2_data.len() as f64;

        println!("{}", mean1);
        println!("{}", mean2);

        Ok(0.0)
    }

    pub async fn get_yearly_data_by_token(
        token: &Token,
    ) -> Result<HashMap<NaiveDate, f64>, anyhow::Error> {
        let method = Method::GET;
        let headers = Self::build_headers();
        let one_year_ago = Self::get_year_ago_date();
        let url = Self::build_url(&token, &one_year_ago);

        let res = Request::process_request(method, url, Some(headers), None).await?;

        if let Some(data) = res["chart"]["result"][0]["indicators"]["quote"][0]["close"].as_array()
        {
            let filtered_data: Vec<f64> = data
                .into_iter()
                .filter_map(|v| v.as_f64()) // Filters out `null` and converts `Value` to `f64`
                .collect();

            let timestamp: Vec<NaiveDate> = res["chart"]["result"][0]["timestamp"]
                .as_array()
                .unwrap()
                .clone()
                .into_iter()
                .map(|v| {
                    DateTime::from_timestamp(v.as_i64().unwrap(), 0)
                        .unwrap()
                        .date_naive()
                })
                .collect();

            let mut final_hashset: HashMap<NaiveDate, f64> = HashMap::new();
            for (i, v) in filtered_data.iter().enumerate() {
                final_hashset.insert(timestamp[i], v.clone());
            }

            Ok(final_hashset)
        } else {
            Err(anyhow!(
                "Not possible to fetch yearly token<{}> data.",
                token.as_string()
            ))
        }
    }

    fn get_year_ago_date() -> DateTime<Local> {
        let now = Local::now();
        let one_year_ago = now - Duration::days(365);

        one_year_ago
    }

    fn build_url(token: &Token, start_date: &DateTime<Local>) -> String {
        format!(
            "
            https://query1.finance.yahoo.com/v8/finance/chart/{}?\
            period1={}&period2={}&interval=1d\
            &includePrePost=true&events=div%7Csplit%7Cearn&&lang=en-US&region=US",
            token.id(),
            start_date.timestamp(),
            Local::now().timestamp()
        )
    }

    fn build_headers() -> HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert("accept", "*/*".parse().unwrap());
        headers.insert("accept-language", "en-US,en;q=0.9".parse().unwrap());
        headers.insert(header::COOKIE, "tbla_id=a5febe28-3e14-4e8a-9825-c65fd3fc6c36-tuctcfc44af; axids=gam=y-v1eCrANE2uJXiPE.3E3uKQDFMVx4Dm0z~A&dv360=eS1lVE94cmsxRTJ1R2x3X3ZGVncxaXFyQjh4MTIxM3FRT35B&ydsp=y-koEQZ3pE2uLoHEGsLWwbqWpXQ.LtQptR~A&tbla=y-RVUe5pxE2uKqZVU.LIHwCi.K6Zc9hDFW~A; GUC=AQEBCAFm1UZnBEIh9QTL&s=AQAAAIRyAHJI&g=ZtP2pw; A1=d=AQABBC20_mUCEBJrnEMPcERJE6Sojvi1WLgFEgEBCAFG1WYEZ6-0b2UB_eMBAAcILbT-Zfi1WLg&S=AQAAAnLpkVOCnX8OdJc3xb1gdhQ; A3=d=AQABBC20_mUCEBJrnEMPcERJE6Sojvi1WLgFEgEBCAFG1WYEZ6-0b2UB_eMBAAcILbT-Zfi1WLg&S=AQAAAnLpkVOCnX8OdJc3xb1gdhQ; A1S=d=AQABBC20_mUCEBJrnEMPcERJE6Sojvi1WLgFEgEBCAFG1WYEZ6-0b2UB_eMBAAcILbT-Zfi1WLg&S=AQAAAnLpkVOCnX8OdJc3xb1gdhQ; cmp=t=1725167265&j=0&u=1---; gpp=DBAA; gpp_sid=-1; _cb=CfOyz-z9nkyD5KVwX; PRF=t%3DBTC-USD%252B%255EGSPC; _cb_svref=https%3A%2F%2Flevenstein.net%2F; _chartbeat2=.1725167265593.1725171315082.1.CaCCrsVFaIABCsBtRB40Rn6D8gESP.2".parse().unwrap());
        headers.insert("dnt", "1".parse().unwrap());
        headers.insert("origin", "https://finance.yahoo.com".parse().unwrap());
        headers.insert("priority", "u=1, i".parse().unwrap());
        headers.insert(
            "referer",
            "https://finance.yahoo.com/quote/BTC-USD/chart/"
                .parse()
                .unwrap(),
        );
        headers.insert(
            "sec-ch-ua",
            "\"Not;A=Brand\";v=\"24\", \"Chromium\";v=\"128\""
                .parse()
                .unwrap(),
        );
        headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
        headers.insert("sec-ch-ua-platform", "\"macOS\"".parse().unwrap());
        headers.insert("sec-fetch-dest", "empty".parse().unwrap());
        headers.insert("sec-fetch-mode", "cors".parse().unwrap());
        headers.insert("sec-fetch-site", "same-site".parse().unwrap());
        headers.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36".parse().unwrap());

        headers
    }
}
