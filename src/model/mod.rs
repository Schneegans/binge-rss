//////////////////////////////////////////////////////////////////////////////////////////
//                           ___ _               ___  ___ ___                           //
//                          | _ |_)_ _  __ _ ___| _ \/ __/ __|                          //
//                          | _ \ | ' \/ _` / -_)   /\__ \__ \                          //
//                          |___/_|_||_\__, \___|_|_\|___/___/                          //
//                                     |___/                                            //
//////////////////////////////////////////////////////////////////////////////////////////

// SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
// SPDX-License-Identifier: MIT

use gtk::glib;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FeedSettings {
  pub title: String,
  pub url: String,
  pub viewed: String,
}

pub struct FeedData {
  pub items: Vec<FeedItemData>,
  pub image: Option<glib::Bytes>,
}

pub struct FeedItemData {
  pub title: String,
  pub url: String,
  pub summary: String,
}
