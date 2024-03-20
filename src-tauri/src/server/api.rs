use actix_web::{post, web};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct TabInfo {
  tabId: Option<i64>,
  url: Option<String>,
  title: Option<String>,
  favIconUrl: Option<String>,
  referrer: Option<String>,
  openerTabId: Option<i64>,
  windowId: Option<i64>,
  timestampMs: i64,
}

#[post("/api/v1/browserlog")]
pub async fn browserlog(tab_info: web::Json<TabInfo>) -> actix_web::Result<String> {
  println!("tab_info: {:?}", tab_info);

  Ok("ok".to_string())
}
