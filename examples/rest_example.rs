use std::collections::HashMap;

use sendgrid_connector::rest::config::SendGridConfig;
use sendgrid_connector::rest::models::EmailAddress;
use sendgrid_connector::rest::rest_client::SendGridRestClient;
use serde_json::json;

#[tokio::main]
async fn main() {
    let app_token = std::env::var("SENDGRID_API_KEY").unwrap();
    let client = SendGridRestClient::new_with_config(app_token, SendGridConfig::test_env());
 
    send_template(&client, 
        std::env::var("SENDGRID_TEMPLATE_ID").unwrap().as_str(), 
        std::env::var("SENDGRID_TO").unwrap().as_str(),
        std::env::var("SENDGRID_TEMPLATE_SUBJECT").unwrap().as_str())
        .await;
 
}

async fn send_template(client: &SendGridRestClient, template_id: &str, email_to: &str, subject: &str) {
    let email_from = std::env::var("SENDGRID_FROM").unwrap();
    let email_cc = std::env::var("SENDGRID_CC").unwrap();
    let email_bcc = std::env::var("SENDGRID_BCC").unwrap();
    let code = std::env::var("SENDGRID_CODE").unwrap();

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
    
    let result = client
    .send_template(
        email_from.as_str(), 
        email_to, 
        email_cc,
        email_bcc, 
        subject, 
        template_id, 
        Some(placeholders)
        )
        .await;

    println!("send_email result: {result:?}");
}
