use gtk::glib;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Feed {
  pub title: String,
  pub url: String,
  pub viewed: String,

  #[serde(skip)]
  pub items: Vec<FeedItem>,

  #[serde(skip)]
  pub image: Option<glib::Bytes>,
}

pub struct FeedItem {
  pub title: String,
  pub url: String,
  pub summary: String,
  pub content: String,
}
