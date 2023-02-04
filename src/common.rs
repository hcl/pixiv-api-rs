use log::{debug, error};
use reqwest::{Response, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Debug, Display};
use std::io::{Cursor, Read};

use crate::client::Session;

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
	debug!("{}", resp_text);
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

impl Urls {
	pub fn replace_ugoira_url(&mut self) {
		let url = self.original.clone();
		let url = url.replace("/img-original/", "/img-zip-ugoira/");
		let url_split: Vec<&str> = url.split("_ugoira0").collect();
		let ret = url_split[0].to_owned() + "_ugoira1920x1080.zip";
		self.original = ret;
	}

	pub async fn save_original(
		&self,
		sess: &Session,
		dst: &String,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let src = self.original.clone();
		let dst_path = std::path::Path::new(dst.as_str());
		let src_url = Url::parse(&src).unwrap();
		let fname = src_url.path_segments().unwrap().last().unwrap();
		let mut r = sess.client.get(&src);
		r = r.header("Referer", "https://www.pixiv.net/");
		match r.send().await {
			Ok(r) => {
				let mut file = std::fs::File::create(dst_path.join(fname))?;
				let mut content = Cursor::new(r.bytes().await?);
				std::io::copy(&mut content, &mut file)?;
				return Ok(());
			}
			Err(e) => {
				error!("error downloading {}: {}", &src, e);
			}
		};
		Ok(())
	}
}
