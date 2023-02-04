use crate::client::{api_header_build, Session};
use crate::common::{parse_response, parse_root_json, BookmarkData, ErrType};

use log::error;

use reqwest::{Response, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Display;
use std::str::FromStr;
use std::{fs::File, io::BufReader, path::Path};

#[derive(Serialize)]
pub struct BookMarkArgs {
	pub offset: i64,
	pub limit: i64,
	pub rest: String,
	pub tag: String,
}

pub enum Catagory {
	Public,
	Private,
}

// impl Catagory {
// 	fn as_str(&self) -> &'static str {
// 		match self {
// 			Catagory::Public => "show",
// 			Catagory::Private => "hide",
// 		}
// 	}
// }

impl Display for Catagory {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Catagory::Public => write!(f, "show"),
			Catagory::Private => write!(f, "hide"),
		}
	}
}

impl Default for BookMarkArgs {
	fn default() -> Self {
		return BookMarkArgs {
			offset: 0,
			limit: 48,
			rest: "show".to_string(),
			tag: "".to_string(),
		};
	}
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmarks {
	pub works: Vec<Work>,
	pub total: i64,
	pub bookmark_tags: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Work {
	id: Value,
	pub title: String,
	pub description: String,
	pub illust_type: i64,
	pub tags: Vec<String>,
	user_id: Value,
	pub user_name: String,
	pub width: i64,
	pub height: i64,
	pub page_count: i64,
	pub bookmark_data: Option<BookmarkData>,
	pub create_date: String,
	pub update_date: String,
	pub is_unlisted: bool,
	pub is_masked: bool,
	pub ai_type: i64,
}

fn parse_id(id: &Value) -> String {
	match id {
		Value::String(s) => return s.to_string(),
		Value::Number(n) => return n.to_string(),
		_ => "".to_string(),
	}
}

impl Work {
	pub fn id(&self) -> String {
		parse_id(&self.id)
	}
	pub fn user_id(&self) -> String {
		parse_id(&self.user_id)
	}
}

#[allow(dead_code)]
pub fn parse_file_path(path: &Path) {
	let input = File::open(path).unwrap();
	let reader = BufReader::new(input);
	let r = parse_root_json(reader).unwrap();
	let body: Bookmarks = serde_json::from_value(r).unwrap();
	for i in body.works {
		println!(
			"id=\"{}\" title=\"{}\" type=\"{}\"",
			i.id(),
			i.title,
			i.illust_type
		);
	}
}

impl Session {
	async fn request_bookmark(&self, args: &BookMarkArgs, page: i64) -> Result<Response, ErrType> {
		let url_str = format!(
			"{}/ajax/user/{}/illusts/bookmarks",
			self.server_url, self.user_info.user_id
		);
		let referer_str = format!(
			"{}/users/{}/bookmarks/artworks?p={}",
			self.server_url, self.user_info.user_id, page
		);

		let hdr = api_header_build(&referer_str, &self.user_info.user_id);

		let url = Url::from_str(url_str.as_str()).unwrap();

		let mut r = self.client.get(url);
		r = r.query(&args);
		r = r.headers(hdr);

		match r.send().await {
			Ok(resp) => return Ok(resp),
			Err(e) => return Err(ErrType::Request(e)),
		};
	}

	pub async fn get_bookmark(
		&self,
		cat: &Catagory,
		tag: &String,
		page: i64,
	) -> Result<Bookmarks, ErrType> {
		assert!(page > 0);
		let args = BookMarkArgs {
			offset: (page - 1) * 48,
			limit: 48,
			rest: cat.to_string(),
			tag: tag.to_string(),
		};
		let resp: Response = match self.request_bookmark(&args, page).await {
			Ok(r) => r,
			Err(e) => {
				error!("get_bookmark->request_bookmark error: {}", e);
				return Err(e);
			}
		};
		let b: Bookmarks = match parse_response(resp).await {
			Ok(r) => r,
			Err(e) => {
				error!("get_bookmark->parse_response error: {}", e);
				return Err(e);
			}
		};
		return Ok(b);
	}
}
