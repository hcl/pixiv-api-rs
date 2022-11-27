use reqwest::Response;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{fmt::Display, io::Read};

#[derive(Debug)]
pub enum ErrType {
	Parser(serde_json::Error),
	Request(reqwest::Error),
	Api(String),
}

impl Display for ErrType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			ErrType::Parser(e) => write!(f, "Parser Error: {}", e.to_string()),
			ErrType::Request(e) => write!(f, "Request Error: {}", e),
			ErrType::Api(e_msg) => write!(f, "API Error: {}", e_msg),
		}
	}
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
	pub error: bool,
	pub message: String,
	pub body: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkData {
	pub id: String,
	pub private: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleCaptionTranslation {
	pub work_title: Value,
	pub work_caption: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Urls {
	pub mini: Option<String>,
	pub thumb: Option<String>,
	pub thumb_mini: Option<String>,
	pub small: String,
	pub regular: String,
	pub original: String,
}

pub fn parse_root_json<R: Read>(rdr: R) -> Result<Value, ErrType> {
	let root: Root = match serde_json::from_reader(rdr) {
		Ok(r) => r,
		Err(e) => return Err(ErrType::Parser(e)),
	};
	if root.error {
		return Err(ErrType::Api(root.message));
	} else {
		return Ok(root.body);
	}
}

pub fn parse_root_json_str(str: &str) -> Result<Value, ErrType> {
	let root: Root = match serde_json::from_str(str) {
		Ok(r) => r,
		Err(e) => return Err(ErrType::Parser(e)),
	};
	if root.error {
		return Err(ErrType::Api(root.message));
	} else {
		return Ok(root.body);
	}
}

pub(crate) async fn parse_response<T: DeserializeOwned>(resp: Response) -> Result<T, ErrType> {
	let resp_text = match resp.text().await {
		Ok(r) => r,
		Err(e) => return Err(ErrType::Request(e)),
	};
	// println!("{}", resp_text);
	let v = match parse_root_json_str(&resp_text) {
		Ok(r) => r,
		Err(e) => return Err(e),
	};
	let v: T = match serde_json::from_value::<T>(v) {
		Ok(v) => v,
		Err(e) => {
			return Err(ErrType::Parser(e));
		}
	};
	return Ok(v);
}
