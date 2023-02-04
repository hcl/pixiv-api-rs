use std::io::{BufReader, Write};
use std::{fs::File, path::Path};

use log::{error, info};
use reqwest::Response;
use reqwest::Url;

use serde::{Deserialize, Serialize};

use crate::client::{api_header_build, Session};
use crate::common::{parse_response, parse_root_json};
use crate::common::{BookmarkData, ErrType, TitleCaptionTranslation, Urls};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Illust {
	pub id: String,
	pub title: String,
	pub description: String,
	pub illust_type: i64,
	pub create_date: String,
	pub upload_date: String,
	pub urls: Urls,
	pub tags: Tags,
	pub alt: String,
	pub user_id: String,
	pub user_name: String,
	pub user_account: String,
	pub like_data: bool,
	pub width: i64,
	pub height: i64,
	pub page_count: i64,
	pub bookmark_count: i64,
	pub like_count: i64,
	pub comment_count: i64,
	pub response_count: i64,
	pub view_count: i64,
	pub is_original: bool,
	pub image_response_count: i64,
	pub is_bookmarkable: bool,
	pub bookmark_data: Option<BookmarkData>,
	pub title_caption_translation: TitleCaptionTranslation,
	pub is_unlisted: bool,
	pub ai_type: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tags {
	pub author_id: String,
	pub is_locked: bool,
	pub tags: Vec<Tag>,
	pub writable: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
	pub tag: String,
	pub locked: bool,
	pub deletable: bool,
	pub user_id: Option<String>,
	pub translation: Option<Translation>,
	pub user_name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Translation {
	pub en: String,
}

#[allow(dead_code)]
pub fn parse_file_path(path: &Path) {
	let input = File::open(path).unwrap();
	let reader = BufReader::new(input);
	let b = parse_root_json(reader).unwrap();
	let body = match serde_json::from_value::<Illust>(b) {
		Ok(body) => body,
		Err(e) => {
			println!("{}", e);
			return;
		}
	};

	println!(
		"id=\"{}\" title=\"{}\" pagecount=\"{}\" bookmark=\"{:?}\"",
		body.id, body.title, body.page_count, body.bookmark_data
	);
	println!(
		"\turl_mini=\"{}\"",
		body.urls.thumb.unwrap_or("".to_string())
	);
}

#[allow(dead_code)]
pub fn clean_json(input: &Path, output: &Path) {
	let input = File::open(input).unwrap();
	let reader = BufReader::new(input);
	let b = parse_root_json(reader).unwrap();
	let b: Illust = serde_json::from_value(b).unwrap();
	let j = serde_json::to_string_pretty(&b).unwrap();
	let mut output = File::create(output).unwrap();
	write!(output, "{}", j).expect("Write error");
}

impl Session {
	async fn request_illust(&self, illust_id: &String) -> Result<Response, ErrType> {
		let url_str = format!("{}/ajax/illust/{}", self.server_url, illust_id);
		let referer_str = format!("{}/artworks/{}", self.server_url, illust_id);
		if self.user_info.user_id.is_none() {
			return Err(ErrType::Call(
				"request_illust->must specify user_id".to_string(),
			));
		}
		let hdr = api_header_build(&referer_str, &self.user_info.user_id);
		let url = Url::parse(&url_str).unwrap();
		let mut r = self.client.get(url);
		r = r.headers(hdr);
		match r.send().await {
			Ok(resp) => return Ok(resp),
			Err(e) => return Err(ErrType::Request(e)),
		};
	}

	pub async fn get_illust(&self, illust_id: &String) -> Result<Illust, ErrType> {
		let resp: Response = match self.request_illust(illust_id).await {
			Ok(r) => r,
			Err(e) => {
				error!("get_illust->request_illust error: {}", e);
				return Err(e);
			}
		};
		let i: Illust = match parse_response(resp).await {
			Ok(r) => r,
			Err(e) => {
				error!("get_illust->parse_response error: {}", e);
				return Err(e);
			}
		};
		return Ok(i);
	}
}

impl Illust {
	pub async fn save(&mut self, sess: &Session, dst: String) -> Result<(), ErrType> {
		let pages = match sess.get_illust_page(&self.id).await {
			Ok(v) => v,
			Err(e) => return Err(e),
		};
		let mut count = 0;
		for mut item in pages {
			info!("Saving {}, page {}/{}", self.id, count, self.page_count - 1);
			if self.illust_type == 2 {
				item.urls.replace_ugoira_url();
				info!("Page {} is ugoira.", count);
			}
			if let Err(_) = item.urls.save_original(&sess, &dst).await {
				continue;
			};
			count += 1;
		}
		Ok(())
	}
}
