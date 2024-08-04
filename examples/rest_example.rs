use std::collections::HashMap;

use sendgrid_connector::rest::config::SendGridConfig;
use sendgrid_connector::rest::models::{EmailAddress, TransactionalTemplate};
use sendgrid_connector::rest::rest_client::SendGridRestClient;
use serde_json::json;

#[tokio::main]
async fn main() {
    let app_token = std::env::var("SENDGRID_API_KEY").unwrap();
    let client = SendGridRestClient::new_with_config(app_token, SendGridConfig::test_env());
    let template = get_template(
        &client, 
        std::env::var("SENDGRID_TEMPLATE_ID").unwrap().as_str()
    ).await;

    if let Some((id, version)) = create_registration_template(&client).await {
        println!("created: {} version: {}", id, version);
        send_template(
            &client,
            id.as_str(), //std::env::var("SENDGRID_TEMPLATE_ID").unwrap().as_str(),
            std::env::var("SENDGRID_TO").unwrap().as_str(),
            std::env::var("SENDGRID_TEMPLATE_SUBJECT").unwrap().as_str(),
        )
        .await;
    }
}

async fn create_registration_template(client: &SendGridRestClient) -> Option<(String, String)> {
    let name = "[en][test] Registration Confirmation";
    match client.create_template(name).await {
        Ok(create_result) => {
            println!("create_template result: {create_result:?}");

            let subject = "Verify Your Email Address for {{company_name}}";
            let html_content = r#"
                    <html>
                    <head>
                    <title></title>
                    </head>
                    <body>
                    <div data-role="module-unsubscribe" class="module" role="module" data-type="unsubscribe" style="color:#444444; font-size:12px; line-height:20px; padding:16px 16px 16px 16px; text-align:Center;" data-muid="4e838cf3-9892-4a6d-94d6-170e474d21e5">
                    <p>Confirm your email</p>
                    <p>Welcome to {{company_name}}. Please confirm your email address using the following activation code: {{code}}</p>
                    <p>If you did not try to register, then ignore this message.</p>
                    </div>
                    </body>
                    </html>
                    "#;

            let plain_content = r#"
                    Confirm your email
                    Welcome to {{company_name}}. Please confirm your email address using the following activation code: {{code}}
                    If you did not try to register, then ignore this message.
                    "#;

            let update_result = client
                .update_template(
                    name,
                    create_result.template_id.as_str(),
                    html_content,
                    plain_content,
                    subject,
                )
                .await;
            println!("update_result result: {update_result:?}");

            return Some((create_result.template_id, update_result.unwrap().id.unwrap()));
        }

        Err(err) => {
            println!("create_template failed: {err:?}");
            None
        }
    }
}

async fn get_template(client: &SendGridRestClient, template_id: &str) -> Option<(TransactionalTemplate)> {
    
    match client.get_template(template_id).await {
        Ok(result) => {
            println!("get_template result: {result:?}");
            return Some((result));
        }

        Err(err) => {
            println!("create_template failed: {err:?}");
            None
        }
    }
}

async fn send_template(
    client: &SendGridRestClient,
    template_id: &str,
    email_to: &str,
    subject: &str,
) {
    let email_from = std::env::var("SENDGRID_FROM").unwrap();
    let email_cc = std::env::var("SENDGRID_CC").unwrap();
    let email_bcc = std::env::var("SENDGRID_BCC").unwrap();
    let code = std::env::var("SENDGRID_CODE").unwrap();
    let company_name: String = std::env::var("SENDGRID_COMPANY").unwrap();

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

    let result = client
        .send_email_by_template(
            email_from.as_str(),
            email_to,
            email_cc,
            email_bcc,
            subject,
            template_id,
            Some(placeholders),
        )
        .await;

    println!("send_email result: {result:?}");
}
