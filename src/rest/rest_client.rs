use std::collections::HashMap;

use crate::rest::config::SendGridConfig;
use crate::rest::endpoints::SendGridEndpoint;
use crate::rest::errors::Error;
use error_chain::bail;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, self};
use reqwest::Response;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::models::*;

#[derive(Clone)]
pub struct SendGridRestClient {
    app_token: String,
    host: String,
    inner_client: reqwest::Client,
}

impl SendGridRestClient {
    pub fn new(app_token: String) -> Self {
        Self::new_with_config(app_token, SendGridConfig::default())
    }

    pub fn new_with_config(app_token: String, config: SendGridConfig) -> Self {
        Self {
            app_token,
            host: config.rest_api_host,
            inner_client: reqwest::Client::new(),
        }
    }

    fn build_headers(&self) -> HeaderMap {
        let mut custom_headers = HeaderMap::new();
        custom_headers.insert(
            header::CONTENT_TYPE, 
            header::HeaderValue::from_static("application/json")
        );

        custom_headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(format!("Bearer {}", &self.app_token).as_str()).unwrap(),
        );

        custom_headers
    }
    
    pub async fn send_template(
        &self,
        email_from: &str,
        email_to: Vec<EmailAddress>,
        email_cc: Vec<EmailAddress>,
        email_bcc: Vec<EmailAddress>,
        subject: &str,
        template_id: &str,
        placeholders: Option<HashMap<String, String>>,
    ) -> Result<SendGridEmailResponse, Error> {

        let email = SendGridEmail {
            from: EmailAddress {
                email: email_from.into(),
                name: None,
            },
            personalizations: vec![Personalization {
                to: email_to,
                cc: Some(email_cc), 
                bcc: Some(email_bcc), 
                dynamic_template_data: placeholders,
            }],
            subject: subject.into(),
            template_id: Some(template_id.to_string()),
        };

        let serialized = serde_json::to_string(&email)?;
        let value: Value = serde_json::from_str(&serialized)?;

        let resp: Option<SendGridEmailResponse> = self
            .post_json(SendGridEndpoint::MailSend, Some(value), None)
            .await?;

        match resp {
            Some(resp) => Ok(resp),
            None => Ok(SendGridEmailResponse::default()),
        }        
    }

    pub async fn post_json<T: DeserializeOwned>(
        &self,
        endpoint: SendGridEndpoint,
        data: Option<serde_json::Value>,
        query_params_string: Option<String>,
    ) -> Result<Option<T>, Error> {
        let url_with_query: String = format!(
            "{}{}{}",
            self.host,
            String::from(endpoint),
            query_params_string.clone().unwrap_or_default()
        );

        let headers = self.build_headers();
        let client = &self.inner_client;
        let response = client
            .post(&url_with_query)
            .headers(headers)
            //.query(&query_params.clone())
            .json(&data.unwrap())
            .send()
            .await?;

        self.handler(response, query_params_string.clone())
            .await
    }

    async fn handler<T: DeserializeOwned>(
        &self,
        response: Response,
        request_json: Option<String>,
    ) -> Result<Option<T>, Error> {
        match response.status() {
            reqwest::StatusCode::OK | reqwest::StatusCode::CREATED | reqwest::StatusCode::ACCEPTED  => {
                let body = response.text().await?;
                if body.trim().is_empty() {
                    // Return None to represent an empty response
                    Ok(None) 
                } else {
                    // Deserialize the response body into type T
                    Ok(Some(serde_json::from_str::<T>(&body)?))
                }
            },
            StatusCode::INTERNAL_SERVER_ERROR => {
                bail!("Internal Server Error");
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                bail!("Service Unavailable");
            }
            StatusCode::UNAUTHORIZED => {
                bail!("Unauthorized");
            }
            StatusCode::BAD_REQUEST => {
                let error = response.text().await?;
                bail!(format!(
                    "Received bad request status. Request: {:?}. Response: {:?}",
                    request_json, error
                ));
            }
            s => {
                let error = response.text().await?;

                bail!(format!("Received response code: {s:?} error: {error:?}"));
            }
        }
    }
}
