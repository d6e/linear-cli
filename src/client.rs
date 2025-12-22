use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::error::{LinearError, Result};

const API_ENDPOINT: &str = "https://api.linear.app/graphql";

pub struct LinearClient {
    http: Client,
    api_key: String,
}

#[derive(Serialize)]
struct GraphQLRequest<'a> {
    query: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize, Debug)]
struct GraphQLError {
    message: String,
}

impl LinearClient {
    pub fn new(api_key: String) -> Self {
        Self {
            http: Client::new(),
            api_key,
        }
    }

    pub async fn query<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
    ) -> Result<T> {
        let request = GraphQLRequest { query, variables };

        let response = self
            .http
            .post(API_ENDPOINT)
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(LinearError::ApiError {
                status: response.status().as_u16(),
                message: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "<failed to read response body>".to_string()),
            });
        }

        let gql_response: GraphQLResponse<T> = response.json().await?;

        if let Some(errors) = gql_response.errors {
            return Err(LinearError::GraphQL {
                messages: errors.into_iter().map(|e| e.message).collect(),
            });
        }

        gql_response.data.ok_or(LinearError::EmptyResponse)
    }
}
