use error_chain::error_chain;
use serde::Deserialize;

#[derive(Default, Debug, PartialEq, Deserialize)]
pub struct SendGridContentError {
    #[serde(rename = "description")]
    pub description: String,
}

error_chain! {
    errors {
        SendGridError(response: SendGridContentError)
    }
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        ReqError(reqwest::Error);
        InvalidHeaderError(reqwest::header::InvalidHeaderValue);
        IoError(std::io::Error);
        ParseFloatError(std::num::ParseFloatError);
        UrlParserError(url::ParseError);
        Json(serde_json::Error);
        TimestampError(std::time::SystemTimeError);
    }
}