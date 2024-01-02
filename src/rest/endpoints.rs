#[derive(Clone)]
pub enum SendGridEndpoint {
    MailSend,
    Templates,
}

impl From<SendGridEndpoint> for String {
    fn from(item: SendGridEndpoint) -> Self {
        String::from(match item {
            SendGridEndpoint::MailSend => "/mail/send",
            SendGridEndpoint::Templates => "/templates",        })
    }
}
