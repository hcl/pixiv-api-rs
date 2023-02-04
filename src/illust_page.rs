use log::error;
use reqwest::{Response, Url};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::client::{api_header_build, Session};
use crate::common::{parse_response, ErrType, Root, Urls};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Illusts {
	pub urls: Urls,
	pub width: i64,
	pub height: i64,
}

#[allow(dead_code)]
pub fn parse_file_path(path: &Path) {
	let input = File::open(path).unwrap();
	let reader = BufReader::new(input);
	let p: Root = serde_json::from_reader(reader).unwrap();
	let b: Vec<Illusts> = serde_json::from_value(p.body).unwrap();
	for item in b {
		println!(
			"thumb_mini=\"{}\" origin_url=\"{}\"",
			item.urls.thumb_mini.unwrap_or("".to_string()),
			item.urls.original
		);
	}
}

impl Session {
	async fn request_illust_page(
		&self,
		illust_id: &String,
		user_id: &String,
	) -> Result<Response, ErrType> {
		let url_str = format!("{}/ajax/illust/{}/pages", self.server_url, illust_id);
		let referer_str = format!("{}/artworks/{}", self.server_url, illust_id);
		let hdr = api_header_build(&referer_str, user_id);
		let url = Url::parse(&url_str).unwrap();
		let mut r = self.client.get(url);
		r = r.headers(hdr);
		match r.send().await {
			Ok(resp) => return Ok(resp),
			Err(e) => return Err(ErrType::Request(e)),
		};
	}

	pub async fn get_illust_page(
		&self,
		illust_id: &String,
		user_id: &String,
	) -> Result<Vec<Illusts>, ErrType> {
		let resp: Response = match self.request_illust_page(illust_id, user_id).await {
			Ok(r) => r,
			Err(e) => {
				error!("request_illust error: {}", e);
				return Err(e);
			}
		};
		let i: Vec<Illusts> = match parse_response(resp).await {
			Ok(r) => r,
			Err(e) => {
				error!("parse_response error: {}", e);
				return Err(e);
			}
		};
		return Ok(i);
	}
}
