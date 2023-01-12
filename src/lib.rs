use macros_make_error::make_error2;
use reqwest::Url;
use serde::*;
use std::fmt::{Debug, Display};

pub use typesense::collection::CollectionResponse;
pub use typesense::field::Field;

pub struct FieldUpdate {
    pub fields: Vec<Field>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MultipleDocumentResponse {
    pub success: bool,
    pub error: String,
    pub document: String,
}

make_error2!(TypesenseError);

pub struct TypesenseClient {
    url: Url,
    api_key: String,
    pub client: reqwest::Client,
}

pub fn create_client(
    url: &str,
    api_key: &str,
) -> Result<TypesenseClient, url::ParseError> {
    let client = reqwest::Client::new();
    Ok(TypesenseClient {
        url: Url::parse(url.as_str())?,
        api_key: api_key.to_string(),
        client,
    })
}

pub async fn create_collection<T>(
    typesense: &TypesenseClient,
    name: &str,
    fields: Vec<Field>,
) -> Result<CollectionResponse, TypesenseError>
where
    T: Serialize,
{
    let cs = typesense::collection::CollectionSchema {
        name: name.to_string(),
        fields,
        default_sorting_field: None,
        token_separators: None,
        symbols_to_index: None,
    };

    let url = typesense
        .url
        .join("collections")
        .map_err(TypesenseError::from_general)?;

    let res = typesense
        .client
        .post(url.as_str())
        .header("X-TYPESENSE-API-KEY", typesense.api_key.as_str())
        .json::<typesense::collection::CollectionSchema>(&cs)
        .send()
        .await
        .map_err(TypesenseError::from_general)?
        .error_for_status()
        .map_err(TypesenseError::from_general)?
        .json::<typesense::collection::CollectionResponse>()
        .await
        .map_err(TypesenseError::from_general);

    res
}

pub async fn patch_collection<T>(
    typesense: &TypesenseClient,
    name: &str,
    fields: Vec<Field>,
) -> Result<FieldUpdate, TypesenseError>
where
    T: Serialize,
{
    let url = typesense
        .url
        .join("collections")
        .join(name)
        .map_err(TypesenseError::from_general)?;

    let res = typesense
        .client
        .patch(url.as_str())
        .header("X-TYPESENSE-API-KEY", typesense.api_key.as_str())
        .json::<FieldUpdate>(&FieldUpdate { fields })
        .send()
        .await
        .map_err(TypesenseError::from_general)?
        .error_for_status()
        .map_err(TypesenseError::from_general)?
        .json::<FieldUpdate>()
        .await
        .map_err(TypesenseError::from_general);

    res
}

pub async fn index_import<'a, T>(
    typesense: &TypesenseClient,
    name: &str,
    documents: Vec<T>,
) -> Result<Vec<MultipleDocumentResponse>, TypesenseError>
where
    T: Serialize + Display + Debug + Deserialize<'a>,
{
    let url = typesense
        .url
        .join("collections")
        .join(name)
        .join("documents")
        .join("import?action=create")
        .map_err(TypesenseError::from_general)?;

    let jsonl = documents
        .iter()
        .fold("".to_string(), |a, b| format!("{}{:?}\n", a, b));

    let res = typesense
        .client
        .post(url.as_str())
        .header("X-TYPESENSE-API-KEY", typesense.api_key.as_str())
        .body(&jsonl)
        .send()
        .await
        .map_err(TypesenseError::from_general)?
        .error_for_status()
        .map_err(TypesenseError::from_general)?
        .text()
        .await
        .map_err(TypesenseError::from_general)
        .and_then(|x| {
            let split = x.split("\n");
            let k = split
                .map(|x| {
                    let t = serde_json::from_str::<MultipleDocumentResponse>(x)
                        .map_err(TypesenseError::from_general);
                    t
                })
                .collect::<Result<_, _>>();
            k
        });

    res
}
