mod compat_tool;

pub use self::compat_tool::parse_compat_tool_mapping;
use derive_more::{Constructor, Display, FromStr};
use futures::{stream, StreamExt};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Constructor, Display, FromStr, Hash, Eq, PartialEq, Debug)]
pub struct AppId(u64);

#[derive(Deserialize, Debug)]
struct AppDetails {
    name: String,
}

#[derive(Deserialize, Debug)]
struct AppDetailsData {
    data: AppDetails,
    success: bool,
}

#[derive(Deserialize, Debug)]
struct AppDetailsResponse(HashMap<u64, AppDetailsData>);

pub async fn fetch_app_names(app_ids: &[&AppId]) -> Result<HashMap<AppId, String>> {
    let urls = app_ids
        .iter()
        .map(|id| {
            format!(
                "https://store.steampowered.com/api/appdetails?filters=basic&appids={}",
                id
            )
        })
        .collect::<Vec<String>>();
    let client = Client::new();

    let names = stream::iter(urls)
        .map(|url| {
            let client = &client;
            async move {
                client
                    .get(url)
                    .send()
                    .await?
                    .json::<AppDetailsResponse>()
                    .await
            }
        })
        .buffer_unordered(10)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter_map(|r| {
            let id = *r.0.keys().next().unwrap();
            let details = r.0.values().into_iter().next().unwrap();
            if details.success {
                Some((AppId::new(id), details.data.name.clone()))
            } else {
                None
            }
        })
        .collect::<HashMap<AppId, String>>();

    Ok(names)
}
