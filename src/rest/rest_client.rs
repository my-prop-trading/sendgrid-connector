use std::collections::HashMap;

use crate::rest::config::SendGridConfig;
use crate::rest::endpoints::SendGridEndpoint;
use crate::rest::errors::Error;
use flurl::*;
use ::http::StatusCode;
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

        let url_with_query = self.get_url_with_headers(endpoint, query_params_string.clone(), url_params_string);

        let body = match data {
            Some(value) => {
                // Serialize `serde_json::Value` to Vec<u8>
                Some(serde_json::to_vec(&value)?)
            }
            None => None,
        };


        let response = url_with_query.post(body).await;
        
        match response {
            Ok(result) => {
                let code = StatusCode::from_u16(result.get_status_code()).unwrap();
                let body = result.receive_body().await.ok();
                println!("Result: {:?}", code);
                match body {
                    Some(body_data) if !body_data.is_empty() => {
                        Ok(Some(serde_json::from_slice(&body_data)?))
                    }
                    _ => {
                        Ok(None)
                    }
                }
            }
            Err(error) => {
                return Err((format!("Received bad request status. Request: {:?}. Response: {:?}", query_params_string, error)).into());
            }
        }
    }

pub async fn get_json<T: DeserializeOwned>(
        &self,
        endpoint: SendGridEndpoint,
        query_params_string: Option<String>,
        url_params_string: Option<String>,
    ) -> Result<Option<T>, Error> {
        let url_with_query = self.get_url_with_headers(endpoint, query_params_string.clone(), url_params_string);

        let response = url_with_query
            .get().await;

        match response {
            Ok(result) => {
                let code = StatusCode::from_u16(result.get_status_code()).unwrap();
                let body = result.receive_body().await.ok();
                println!("Result: {:?}", code);
                match body {
                    Some(body_data) if !body_data.is_empty() => {
                        Ok(Some(serde_json::from_slice(&body_data)?))
                    }
                    _ => {
                        Ok(None)
                    }
                }
            }
            Err(error) => {
                return Err((format!("Received bad request status. Request: {:?}. Response: {:?}", query_params_string, error)).into());
            }
        }
    }

    pub fn get_url_with_headers(
        &self,
        endpoint: SendGridEndpoint,
        query_params_string: Option<String>,
        url_params_string: Option<String>,
    ) -> FlUrl{
        let url_with_query: String = format!(
            "{}{}{}{}",
            self.host,
            String::from(endpoint),
            url_params_string.clone().unwrap_or_default(),
            query_params_string.clone().unwrap_or_default()
        );

        let url_with_query = url_with_query.with_header("CONTENT_TYPE", "application/json");
        let url_with_query = url_with_query.with_header("authorization", format!("Bearer {}", &self.app_token).as_str());
        return url_with_query; 
    }
}

#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use flurl::*;

    use crate::rest::config::SendGridConfig;

    #[tokio::test]
    #[ignore]
    async fn test_get_click() {
        dotenv().ok();
        let api_key = std::env::var("SENDGRID_API_KEY").unwrap();

        match FlUrl::new(SendGridConfig::default().rest_api_host)
            .do_not_reuse_connection()
            .with_header("X-Api-Key", &api_key)
            .get()
            .await
        {
            Ok(result) => {
                let body = result.receive_body().await.unwrap();
                let parsed = String::from_utf8(body).unwrap();  

                println!("Response: {:?}", parsed_result);
            }
            Err(err) => {
                println!("Error: {}", err.to_string());
            }
        };
        assert!(false)
    }
}