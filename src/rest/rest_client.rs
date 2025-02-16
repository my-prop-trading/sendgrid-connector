use std::collections::HashMap;

use crate::rest::config::SendGridConfig;
use crate::rest::endpoints::SendGridEndpoint;
use crate::rest::errors::Error;
use error_chain::bail;
use flurl::*;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::models::*;

#[derive(Clone)]
pub struct SendGridRestClient {
    app_token: String,
    host: String
}

impl SendGridRestClient {
    pub fn new(app_token: String, rest_api_host: Option<String>) -> Self {

        Self::new_with_config(
            app_token, 
            if let Some(rest_api_host) = rest_api_host { SendGridConfig {rest_api_host}} else { SendGridConfig::default() })
    }

    pub fn new_with_config(app_token: String, config: SendGridConfig) -> Self {
        Self {
            app_token,
            host: config.rest_api_host
        }
    }

    pub fn build_headers(&self, url: String) -> FlUrl {
        let url = url.with_header("CONTENT_TYPE", "application/json");
        let url = url.with_header("authorization", format!("Bearer {}", &self.app_token).as_str());

        return url;
    }

    pub async fn send_email_by_template(
        &self,
        email_from: &str,
        email_from_name: Option<&str>,
        email_to: Vec<EmailAddress>,
        email_cc: Option<Vec<EmailAddress>>,
        email_bcc: Option<Vec<EmailAddress>>,
        subject: &str,
        template_id: &str,
        placeholders: Option<HashMap<String, String>>,
    ) -> Result<SendGridEmailResponse, Error> {
        let email = SendGridEmail {
            from: EmailAddress {
                email: email_from.into(),
                name: email_from_name.map(|name| name.to_string()),
            },
            personalizations: vec![Personalization {
                to: email_to,
                cc: email_cc,
                bcc: email_bcc,
                dynamic_template_data: placeholders,
            }],
            subject: subject.into(),
            template_id: Some(template_id.to_string()),
        };

        let serialized = serde_json::to_string(&email)?;
        let value: Value = serde_json::from_str(&serialized)?;

        let resp: Option<SendGridEmailResponse> = self
            .post_json(SendGridEndpoint::MailSend, Some(value), None, None)
            .await?;

        match resp {
            Some(resp) => Ok(resp),
            None => Ok(SendGridEmailResponse::default()),
        }
    }

    pub async fn create_template(
        &self,
        name: &str,
    ) -> Result<CreateSendGridTemplateResponse, Error> {
        let email = CreateSendGridTemplate {
            name: name.to_string(),
            generation: "dynamic".to_string(),
        };

        let serialized = serde_json::to_string(&email)?;
        let value: Value = serde_json::from_str(&serialized)?;

        let resp: Option<TransactionalTemplate> = self
            .post_json(SendGridEndpoint::Templates, Some(value), None, None)
            .await?;

        match resp {
            Some(resp) => Ok(CreateSendGridTemplateResponse { 
                template_id: resp.id,
            }),
            None => Ok(CreateSendGridTemplateResponse::default()),
        } 
    }

    pub async fn get_template(&self, template_id: &str) -> Result<Option<TransactionalTemplate>, Error> {
        let resp: Option<TransactionalTemplate> = self
            .get_json(
                SendGridEndpoint::Templates,
                Some(format!("/{}", template_id)),
                None,
            )
            .await?;

        Ok(resp)
    }

    pub async fn update_template(
        &self,
        name: &str,
        template_id: &str,
        html_content: &str,
        plain_content: &str,
        subject: &str,
    ) -> Result<Option<TransactionalTemplateVersion>, Error> {
        let request = SendGridTemplateVersionRequest {
            template_id: template_id.to_string(),
            active: Some(1),
            name: name.to_string(),
            html_content: Some(html_content.to_string()),
            plain_content: Some(plain_content.to_string()),
            generate_plain_content: Some(true),
            subject: subject.to_string(),
            editor: Some("code".to_string()),
            test_data: None,
        };

        let serialized = serde_json::to_string(&request)?;
        let value: Value = serde_json::from_str(&serialized)?;
        println!("{:?}", value);
        let resp: Option<TransactionalTemplateVersion> = self
            .post_json(
                SendGridEndpoint::Templates,
                Some(value),
                None,
                Some(format!("/{}/versions", template_id)),
            )
            .await?;

        match resp {
            Some(resp) => Ok(Some(resp)),
            None => Ok(None),
        }
    }

    pub async fn post_json<T: DeserializeOwned>(
        &self,
        endpoint: SendGridEndpoint,
        data: Option<serde_json::Value>,
        query_params_string: Option<String>,
        url_params_string: Option<String>,
    ) -> Result<Option<T>, Error> {

        let url_with_query: String = format!(
            "{}{}{}{}",
            self.host,
            String::from(endpoint),
            url_params_string.clone().unwrap_or_default(),
            query_params_string.clone().unwrap_or_default()
        );

        let body = match data {
            Some(value) => {
                // Serialize `serde_json::Value` to Vec<u8>
                Some(serde_json::to_vec(&value)?)
            }
            None => None,
        };
        
        let url = self.build_headers(url_with_query);

        let response = url
            .post(body).await;

        match response {
            Ok(result) => self.handler(result, query_params_string.clone()).await,
            Err(_) => todo!(),
        }
    }

pub async fn get_json<T: DeserializeOwned>(
        &self,
        endpoint: SendGridEndpoint,
        query_params_string: Option<String>,
        url_params_string: Option<String>,
    ) -> Result<Option<T>, Error> {
        let url_with_query: String = format!(
            "{}{}{}{}",
            self.host,
            String::from(endpoint),
            url_params_string.clone().unwrap_or_default(),
            query_params_string.clone().unwrap_or_default()
        );

        let url = self.build_headers(url_with_query);

        let response = url
            .get().await;

        match response {
            Ok(result) => self.handler(result, query_params_string.clone()).await,
            Err(_) => todo!(),
        }
    }

    async fn handler<T: DeserializeOwned>(
        &self,
        response: FlUrlResponse,
        request_json: Option<String>,
    ) -> Result<Option<T>, Error> {
        match response.get_status_code() {
            200 | 201 | 202 => {
                let body = response.receive_body().await;
                match  body {
                    Ok(result) => {
                            if result.is_empty() {
                                // Return None to represent an empty response
                                Ok(None)
                            } else {
                                // Deserialize the response body into type T
                                Ok(Some(serde_json::from_slice(&result)?))
                            }
                        }
                    Err(_) => todo!(),
                    }
                }
            400 => {
                let body = response.receive_body().await;
                match  body {
                    Ok(result) => {
                        Ok(Some(serde_json::from_slice(&result)?))
                    }
                    Err(error) => {
                        return Err((format!("Received bad request status. Request: {:?}. Response: {:?}",request_json, error)).into());
                        },
                    }
                }
            401 => {
                bail!("Unauthorized");
            }

            500 => {
                bail!("Internal Server Error");
            }
           503 => {
                bail!("Service Unavailable");
            }

            s => {
                let body = response.receive_body().await;
                match  body {
                    Ok(result) => {
                        Ok(Some(serde_json::from_slice(&result)?))
                    }
                    Err(error) => {
                            bail!(format!("Received response code: {s:?} error: {error:?}"));
                        },
                    }
            }
        }
    }

    pub async fn handle_error<T: DeserializeOwned>(
        response: FlUrlResponse,
        message: String
    ) -> Result<Option<T>, Error> {
        let body = response.receive_body().await;
            match  body {
                Ok(result) => {
                    Ok(Some(serde_json::from_slice(&result)?))
                }
                Err(_) => {
                    return Err((message).into());
                    },
                }
    }
}
