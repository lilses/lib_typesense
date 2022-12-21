use my_macros::make_error;
use reqwest::Url;
use serde::Serialize;
use std::fmt::Display;

make_error!(TypesenseError);

pub struct TypesenseClient {
    pub url: Url,
    pub client: reqwest::Client,
}

pub fn create_client(url: &str) -> Result<TypesenseClient, url::ParseError> {
    let client = reqwest::Client::new();
    Ok(TypesenseClient {
        url: Url::parse(url.as_str())?,
        client,
    })
}

pub async fn create_collection<T>(
    typesense: &TypesenseClient,
    collection: T,
) where
    T: Serialize,
{
    let url = client.url.join("collections")?;

    typesense
        .client
        .post(url.to_string())
        .json::<T>(collection)
        .send()
        .await
        .map_err(MyError::from_general)?
        .error_for_status()
        .map_err(MyError::from_general)?
        .text()
        .await
        .map_err(MyError::from_general)?;
}
