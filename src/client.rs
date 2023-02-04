// use cookie::time::{Duration, OffsetDateTime};
use cookie::Cookie as RawCookie;
use cookie_store::Cookie as WrappedCookie;
use reqwest::header::{HeaderMap, HeaderValue};

use reqwest::Client;
use reqwest::Url;
use reqwest_cookie_store::CookieStore;
use reqwest_cookie_store::CookieStoreMutex;
use std::fs;
use std::io::BufReader;
use std::sync::Arc;

pub struct Session {
	pub user_info: UserInfo,
	pub server_url: String,
	pub client: Client,
	pub cookie_jar: Arc<CookieStoreMutex>,
}

pub struct UserInfo {
	pub user_id: Option<String>,
}

static DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36";
static DEFAULT_URL: &str = "https://www.pixiv.net";

impl Session {
	pub fn new(user_id: Option<String>) -> Session {
		let mut cb = Client::builder();
		cb = cb.user_agent(DEFAULT_USER_AGENT);

		let j = new_cookie_jar();
		cb = cb.cookie_provider(j.clone());
		let c = match cb.build() {
			Ok(c) => c,
			Err(e) => panic!("{}", e),
		};
		return Session {
			user_info: UserInfo { user_id },
			server_url: DEFAULT_URL.to_string(),
			client: c,
			cookie_jar: j,
		};
	}

	pub fn add_cookie_str(&self, cookie_str: &'static str, url: &Url) {
		let c = RawCookie::parse(cookie_str).unwrap();
		// c.set_http_only(true);
		// c.set_expires(OffsetDateTime::now_utc() + Duration::weeks(52));
		let c = WrappedCookie::try_from_raw_cookie(&c, url).unwrap();
		self.cookie_jar.lock().unwrap().insert(c, url).unwrap();
	}

	pub fn add_cookie_string(&self, cookie_str: String, url: &Url) {
		let c = RawCookie::parse(cookie_str).unwrap();
		// c.set_http_only(true);
		// c.set_expires(OffsetDateTime::now_utc() + Duration::weeks(52));
		let c = WrappedCookie::try_from_raw_cookie(&c, url).unwrap();
		self.cookie_jar.lock().unwrap().insert(c, url).unwrap();
	}

	pub fn load_cookie(&self, path: &str) {
		let file = match fs::OpenOptions::new().read(true).open(path) {
			Ok(f) => f,
			Err(e) => panic!("{}", e),
		};
		let buf = BufReader::new(file);
		let mut store = self.cookie_jar.lock().unwrap();
		let cs = match CookieStore::load_json_all(buf) {
			Ok(c) => c,
			Err(e) => panic!("{}", e),
		};
		store.clone_from(&cs);
	}

	pub fn save_cookie(&self, path: &str) {
		let mut file = match fs::OpenOptions::new()
			.write(true)
			.truncate(true)
			.create(true)
			.open(path)
		{
			Ok(f) => f,
			Err(e) => panic!("{}", e),
		};
		let store = self.cookie_jar.lock().unwrap();
		if let Err(e) = store.save_incl_expired_and_nonpersistent_json(&mut file) {
			panic!("{}", e);
		}
	}
}

fn new_cookie_jar() -> Arc<CookieStoreMutex> {
	let jar = CookieStore::default();
	let j = Arc::new(CookieStoreMutex::new(jar));
	return j;
}

pub(crate) fn api_header_build(referer_str: &String, user_id: &Option<String>) -> HeaderMap {
	let mut hdr = HeaderMap::new();
	hdr.insert("authority", HeaderValue::from_static("www.pixiv.net"));
	hdr.insert("accept", HeaderValue::from_static("application/json"));
	hdr.insert(
		"accept-language",
		HeaderValue::from_static("en-US,en;q=0.5"),
	);
	hdr.insert(
		"referer",
		HeaderValue::from_str(referer_str.as_str()).unwrap(),
	);
	if let Some(uid) = user_id {
		hdr.insert("x-user-id", HeaderValue::from_str(uid.as_str()).unwrap());
	};
	return hdr;
}
