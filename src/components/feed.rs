use gtk::glib;

pub struct Feed {
  pub items: Vec<FeedItem>,
  pub image: Option<glib::Bytes>,
}

pub struct FeedItem {
  pub title: String,
  pub url: String,
  pub content: String,
}
