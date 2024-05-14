use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EmailAddress {
    pub email: String,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Personalization {
    pub to: Vec<EmailAddress>,
    #[serde(rename = "dynamic_template_data", skip_serializing_if = "Option::is_none")]
    pub dynamic_template_data: Option<HashMap<String, String>>, 
    pub cc: Option<Vec<EmailAddress>>,
    pub bcc: Option<Vec<EmailAddress>>,
}

#[derive(Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct SendGridEmail {
    #[serde(rename = "from")]
    pub from: EmailAddress,
    #[serde(rename = "personalizations")]
    pub personalizations: Vec<Personalization>,
    #[serde(rename = "template_id", skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    #[serde(rename = "subject")]
    pub subject: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SendGridEmailResponse {

}

#[derive(Debug, serde::Serialize)]
pub struct CreateSendGridTemplate {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "generation")]
    pub generation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreateSendGridTemplateResponse {
    pub template_id: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TransactionalTemplate {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "generation")]
    pub generation: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "versions")]
    pub versions: Vec<TransactionalTemplateVersion>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TransactionalTemplateVersion {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "template_id", skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    #[serde(rename = "active", skip_serializing_if = "Option::is_none")]
    pub active: Option<i32>,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "subject")]
    pub subject: String,
    #[serde(rename = "updated_at", skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(rename = "generate_plain_content", skip_serializing_if = "Option::is_none")]
    pub generate_plain_content: Option<bool>,
    #[serde(rename = "html_content", skip_serializing_if = "Option::is_none")]
    pub html_content: Option<String>,
    #[serde(rename = "plain_content", skip_serializing_if = "Option::is_none")]
    pub plain_content: Option<String>,
    #[serde(rename = "editor", skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    #[serde(rename = "thumbnail_url", skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(rename = "warning", skip_serializing_if = "Option::is_none")]
    pub warning: Option<Warning>,
    #[serde(rename = "test_data", skip_serializing_if = "Option::is_none")]
    pub test_data: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SendGridTemplateVersionRequest {
    #[serde(rename = "template_id")]
    pub template_id: String,
    #[serde(rename = "active", skip_serializing_if = "Option::is_none")]
    pub active: Option<i32>,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "html_content", skip_serializing_if = "Option::is_none")]
    pub html_content: Option<String>,
    #[serde(rename = "plain_content", skip_serializing_if = "Option::is_none")]
    pub plain_content: Option<String>,
    #[serde(rename = "generate_plain_content", skip_serializing_if = "Option::is_none")]
    pub generate_plain_content: Option<bool>,
    #[serde(rename = "subject")]
    pub subject: String,
    #[serde(rename = "editor", skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    #[serde(rename = "test_data", skip_serializing_if = "Option::is_none")]
    pub test_data: Option<String>,
}


// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// pub struct SendGridTemplateVersionResponse {
//     #[serde(rename = "id")]
//     pub id: String,
//     #[serde(rename = "template_id")]
//     pub template_id: String,
//     #[serde(rename = "active")]
//     pub active: i32,
//     #[serde(rename = "name")]
//     pub name: String,
//     #[serde(rename = "subject")]
//     pub subject: String,
//     #[serde(rename = "updated_at")]
//     pub updated_at: String,
//     #[serde(rename = "generate_plain_content")]
//     pub generate_plain_content: bool,
//     #[serde(rename = "html_content")]
//     pub html_content: String,
//     #[serde(rename = "plain_content")]
//     pub plain_content: String,
//     #[serde(rename = "editor")]
//     pub editor: String,
//     #[serde(rename = "thumbnail_url")]
//     pub thumbnail_url: Option<String>,
//     #[serde(rename = "warnings")]
//     pub warnings: Option<Vec<Warning>>,

//     #[serde(rename = "test_data")]
//     pub test_data: Option<String>,

// }

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Warning {
    #[serde(rename = "message")]
    pub message: String,
}
