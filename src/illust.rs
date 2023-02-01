use std::io::{BufReader, Cursor, Write};
use std::{fs::File, path::Path};

use log::error;
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
	)
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
	async fn request_illust(
		&self,
		illust_id: &String,
		user_id: &String,
	) -> Result<Response, ErrType> {
		let url_str = format!("{}/ajax/illust/{}", self.server_url, illust_id);
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

	pub async fn get_illust(
		&self,
		illust_id: &String,
		user_id: &String,
	) -> Result<Illust, ErrType> {
		let resp: Response = match self.request_illust(illust_id, user_id).await {
			Ok(r) => r,
			Err(e) => {
				error!("request_illust error: {}", e);
				return Err(e);
			}
		};
		let i: Illust = match parse_response(resp).await {
			Ok(r) => r,
			Err(e) => {
				error!("parse_response error: {}", e);
				return Err(e);
			}
		};
		return Ok(i);
	}
}

impl Illust {
	async fn save_illust(&self, sess: &Session, src: String, dst: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

	pub async fn save(&self, sess: &Session, dst: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let src = self.urls.original.clone();
		match self.illust_type {
			2 => {
				let src = src.replace("/img-original/", "/img-zip-ugoira/");
				let src = src.split("_ugoira0");
				let tmp_vec: Vec<&str> = src.collect();
				let src = tmp_vec[0].to_owned() + "_ugoira1920x1080.zip";
				self.save_illust(sess, src, dst).await?;
			},
			_ => {
				self.save_illust(sess, src, dst).await?;
			}
		}
		Ok(())
	}
}
