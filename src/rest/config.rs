#[derive(Clone, Debug)]
pub struct SendGridConfig {
    pub rest_api_host: String,
}

impl Default for SendGridConfig {
    fn default() -> Self {
        Self {
            rest_api_host: "https://api.sendgrid.com/v3/".into(),
        }
    }
}

impl SendGridConfig {
    pub fn test_env() -> Self {
        Self {
            rest_api_host: "https://api.sendgrid.com/v3/".into(),
        }
    }
}
