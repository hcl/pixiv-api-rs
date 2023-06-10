use log::{error, info};
use reqwest::Url;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use reqwest::header::{HeaderMap, HeaderValue};

use crate::client::DEFAULT_URL;
use crate::common::Urls;
use crate::{client::Session, illust::Illust, illust_page::Illusts};

pub(crate) fn download_header_build() -> HeaderMap {
	let mut hdr = HeaderMap::new();
	hdr.insert("accept", HeaderValue::from_static("*/*"));
	hdr.insert(
		"accept-language",
		HeaderValue::from_static("en-US,en;q=0.5"),
	);
	hdr.insert("referer", HeaderValue::from_str(DEFAULT_URL).unwrap());
	return hdr;
}

pub(crate) fn ugoira_url_parse(orig: &String) -> String {
	let orig = orig.clone();
	let replaced = orig.replace("/img-original/", "/img-zip-ugoira/");
	let split_vec: Vec<&str> = replaced.split('_').collect();
	let parsed = format!("{}_ugoira1920x1080.zip", split_vec[0]);
	return parsed;
}

impl Urls {
	pub async fn download_original(&self, sess: &Session, dest: &PathBuf) {
		let url = self.original.clone();
		let mut dest = dest.clone();
		let fname = Url::parse(&url)
			.unwrap()
			.path_segments()
			.unwrap()
			.last()
			.unwrap()
			.to_string();
		dest.push(fname);
		info!("{} -> {}", url, dest.display());
		let mut r = sess.client.get(&url);
		r = r.headers(download_header_build());
		let resp = match r.send().await {
			Ok(r) => r,
			Err(e) => {
				error!("{}", e);
				return;
			}
		};
		let resp = match resp.bytes().await {
			Ok(r) => r,
			Err(e) => {
				error!("{}", e);
				return;
			}
		};
		let mut dest_file = match File::create(dest) {
			Ok(f) => f,
			Err(e) => {
				error!("{}", e);
				return;
			}
		};
		match dest_file.write_all(&resp) {
			Ok(_) => (),
			Err(e) => {
				error!("{}", e);
			}
		};
	}
}

impl Illust {
	pub async fn download(&self, sess: &Session, dest: &PathBuf) {
		let mut urls = self.urls.clone();
		if self.illust_type == 2 {
			urls.original = ugoira_url_parse(&urls.original);
		}
		urls.download_original(sess, dest).await;
	}
}

impl Illusts {
	pub async fn download(&self, sess: &Session, dest: &PathBuf) {
		for item in &self.0 {
			item.urls.download_original(sess, dest).await;
		}
	}
}
