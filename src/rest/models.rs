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
    pub dynamic_template_data: Option<serde_json::Value>, // Or you can create a specific struct for dynamic data
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
    #[serde(rename = "content")]
    pub content: Vec<Content>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SendGridEmailResponse {

}