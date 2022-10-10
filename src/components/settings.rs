use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FeedSettings {
  pub title: String,
  pub url: String,
}
