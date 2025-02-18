use std::collections::HashMap;

use crate::rest::config::SendGridConfig;
use crate::rest::endpoints::SendGridEndpoint;
use crate::rest::errors::Error;
use flurl::*;
use ::http::StatusCode;
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

    pub async fn send_email_from_template(
        &self,
        email_from: EmailAddress,
        email_to: Vec<EmailAddress>,
        email_cc: Option<Vec<EmailAddress>>,
        email_bcc: Option<Vec<EmailAddress>>,
        template_id: &str,
        placeholders: Option<HashMap<String, String>>,
    ) -> Result<String, String> {
        let sg_email = SendGridEmail {
            from: email_from.into(),
            personalizations: vec![Personalization {
                to: email_to.into_iter().map(|item| item.into()).collect(),
                cc: email_cc,
                bcc: email_bcc,
                dynamic_template_data: placeholders,
            }],
            template_id: Some(template_id.to_string()),
            subject: String::new()
        };

        let payload = serde_json::to_value(&sg_email)
            .map_err(|e| format!("Failed to serialize email data into JSON bytes: {}", e))?;

        let client = FlUrl::new(self.host.clone())
            .append_path_segment(String::from(SendGridEndpoint::MailSend))
            .with_header("Content-Type", "application/json")
            .with_header("Authorization", format!("Bearer {}", self.app_token));

        if std::env::var("DEBUG").is_ok() {
            println!("{:?}", client.url.to_string());
            println!("{:?}", &payload);
        }

        let response = client
            .post_json(&payload)
            .await
            .map_err(|e| format!("HTTP POST failed: {:?}", e))?;

        let code = StatusCode::from_u16(response.get_status_code())
            .map_err(|e| format!("Failed to read status result: {:?}", e))?;

        if code == StatusCode::ACCEPTED {
            let msg = format!("Successfully sent template: '{}'", template_id);
            return Ok(msg);
        }

        let body = response
            .receive_body()
            .await
            .map_err(|e| format!("Failed to receive body: {:?}", e))?;
        let parsed = String::from_utf8(body)
            .map_err(|e| format!("Failed to convert from_utf8 body: {}", e))?;

        let msg = format!(
            "Failed to sent template '{}'. Response status: {:?}. Message: {}",
            template_id, code, parsed
        );
        Err(msg)
    }

    pub async fn create_template(
        &self,
        name: &str,
    ) -> Result<CreateSendGridTemplateResponse, String> {
        let email = CreateSendGridTemplate {
            name: name.to_string(),
            generation: "dynamic".to_string(),
        };

        let serialized = serde_json::to_string(&email)
            .map_err(|e| format!("Failed to convert from_utf8 body: {}", e))?;
        let value: Value = serde_json::from_str(&serialized)
        .map_err(|e| format!("Failed to convert from_utf8 body: {}", e))?;

        let client = FlUrl::new(self.host.clone())
            .append_path_segment(String::from(SendGridEndpoint::Templates))
            .with_header("Content-Type", "application/json")
            .with_header("Authorization", format!("Bearer {}", self.app_token));

        if std::env::var("DEBUG").is_ok() {
            println!("{:?}", client.url.to_string());
            println!("{:?}", &value);
        }

        let response = client
            .post_json(&value)
            .await
            .map_err(|e| format!("HTTP POST failed: {:?}", e))?;

        let code = StatusCode::from_u16(response.get_status_code())
            .map_err(|e| format!("Failed to read status result: {:?}", e))?;

        if code == StatusCode::ACCEPTED {
            let body = response.receive_body()
            .await
            .map_err(|e| format!("Failed to receive body: {:?}", e))?;

            let parsed_response: CreateSendGridTemplateResponse =
                serde_json::from_slice(&body)
            .map_err(|e| format!("Failed to receive body: {:?}", e))?;  // Deserialize the body into CreateSendGridTemplateResponse

            // Return the parsed response wrapped in Some()
            return Ok(parsed_response);
        }

        let body = response
            .receive_body()
            .await
            .map_err(|e| format!("Failed to receive body: {:?}", e))?;
        let parsed = String::from_utf8(body)
            .map_err(|e| format!("Failed to convert from_utf8 body: {}", e))?;

        let msg = format!(
            "Failed to sent template '{}'. Response status: {:?}. Message: {}",
            name, code, parsed
        );
        Err(msg)
    }
 
    pub async fn get_template(
        &self,
        template_id: &str
    ) -> Result<Option<TransactionalTemplate>, Error> {
        let client = FlUrl::new(self.host.clone())
            .append_path_segment(String::from(SendGridEndpoint::Templates))
            .with_header("Content-Type", "application/json")
            .with_header("Authorization", format!("Bearer {}", self.app_token));

        if std::env::var("DEBUG").is_ok() {
            println!("{:?}", client.url.to_string());
            println!("{:?}", &template_id);
        }

        let response = client
            .get()
            .await
            .map_err(|e| format!("HTTP GET failed: {:?}", e))?;

        let code: StatusCode = StatusCode::from_u16(response.get_status_code())
            .map_err(|e| format!("Failed to read status result: {:?}", e))?;

        let body = response
            .receive_body()
            .await
            .map_err(|e| format!("Failed to receive body: {:?}", e))?;
        if code == StatusCode::OK {
            let template: TransactionalTemplate = serde_json::from_slice(&body)
            .map_err(|e| format!("Failed to deserialize body: {:?}", e))?;
            return Ok(Some(template));
        }

        let parsed = String::from_utf8(body)
            .map_err(|e| format!("Failed to convert from_utf8 body: {}", e))?;

        let msg = format!(
            "Failed to sent template '{}'. Response status: {:?}. Message: {}",
            template_id, code, parsed
        );
        Err(msg.into())
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

        let template_version = TransactionalTemplateVersion {
        id: Some("123".to_string()),
        template_id: Some("template-123".to_string()),
        active: Some(1),
        name: "Test Template".to_string(),
        subject: "Test Subject".to_string(),
        updated_at: Some("2025-02-18T12:00:00Z".to_string()),
        generate_plain_content: Some(true),
        html_content: Some("<html><body>Test</body></html>".to_string()),
        plain_content: Some("Test plain content".to_string()),
        editor: Some("html-editor".to_string()),
        thumbnail_url: None,
        warning: Some(Warning {
            message: "This is a warning".to_string(),
        }),
        test_data: Some("test-data".to_string()),
    };
        return Ok(Some(template_version));
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use dotenv::dotenv;

    use crate::rest::{models::EmailAddress, rest_client::SendGridRestClient};

    #[tokio::test]
    #[ignore]
    async fn test_mail_send_by_template_id() {
        dotenv().ok();
        let sendgrid_api_key = std::env::var("SENDGRID_API_KEY").unwrap();
        let email_to = std::env::var("SENDGRID_TO").unwrap();
        let email_from = std::env::var("SENDGRID_FROM").unwrap();
        let email_from_name = std::env::var("SENDGRID_FROM_NAME").unwrap();
        let email_cc = std::env::var("SENDGRID_CC").unwrap();
        let email_bcc = std::env::var("SENDGRID_BCC").unwrap();
        let code = std::env::var("SENDGRID_CODE").unwrap();
        let company_name = std::env::var("SENDGRID_COMPANY").unwrap();
        let template_id = std::env::var("SENDGRID_TEMPLATE_ID").unwrap();

        let email_from = EmailAddress {
            email: email_from.into(),
            name: Some(email_from_name),
        };

        let email_to = vec![EmailAddress {
            email: email_to.into(),
            name: None,
        }];

        let email_cc = vec![EmailAddress {
            email: email_cc.into(),
            name: None,
        }];

        let email_bcc = vec![EmailAddress {
            email: email_bcc.into(),
            name: None,
        }];

        let mut placeholders = HashMap::new();
        placeholders.insert("code".to_string(), code);
        placeholders.insert("company_name".to_string(), company_name);

        let client = SendGridRestClient::new(sendgrid_api_key, None);
        match client
            .send_email_from_template(
                email_from,
                email_to,
                Some(email_cc),
                Some(email_bcc),
                &template_id,
                Some(placeholders),
            )
            .await
        {
            Ok(msg) => {
                println!("Sent {:?}", msg);
                assert!(true)
            }
            Err(err) => {
                println!("Failed: {:?}", err);
                assert!(false)
            }
        }
    }

    
    #[tokio::test]
    #[ignore]
    async fn test_mail_get_template() {
        dotenv().ok();
        let sendgrid_api_key = std::env::var("SENDGRID_API_KEY").unwrap();
        let template_id = std::env::var("SENDGRID_TEMPLATE_ID").unwrap();

        let client = SendGridRestClient::new(sendgrid_api_key, None);
        match client
            .get_template(&template_id)
            .await
        {
            Ok(msg) => {
                println!("Sent {:?}", msg);
                assert!(true)
            }
            Err(err) => {
                println!("Failed: {:?}", err);
                assert!(false)
            }
        }
    }
}