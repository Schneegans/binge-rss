// ------------------------------------------------------------------------------------ //
//                           ___ _               ___  ___ ___                           //
//                          | _ |_)_ _  __ _ ___| _ \/ __/ __|                          //
//                          | _ \ | ' \/ _` / -_)   /\__ \__ \                          //
//                          |___/_|_||_\__, \___|_|_\|___/___/                          //
//                                     |___/                                            //
// ------------------------------------------------------------------------------------ //

// SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
// SPDX-License-Identifier: MIT

use gtk::{glib, prelude::*, subclass::prelude::*};
use once_cell::sync::Lazy;
use std::cell::RefCell;

// ---------------------------------------------------------------------------------------
// A FeedItem is a very simple GObject with two string properties: A title and an URL. It
// is used to populate the feed item lists in the user interface.
glib::wrapper! {
  pub struct FeedItem(ObjectSubclass<imp::FeedItem>);
}

impl FeedItem {
  // ----------------------------------------------------------------- constructor methods

  pub fn new(title: String, url: String) -> Self {
    glib::Object::builder()
      .property("title", &title)
      .property("url", &url)
      .build()
  }

  // ---------------------------------------------------------------------- public methods

  // Get the title of the FeedItem. This should be shown to the user.
  pub fn get_title(&self) -> String {
    self.imp().title.borrow().clone()
  }

  // Get the URL of the FeedItem. This should be opened when the item is activated.
  pub fn get_url(&self) -> String {
    self.imp().url.borrow().clone()
  }
}

mod imp {
  use super::*;

  // -------------------------------------------------------------------------------------

  // Object holding the internal state of a FeedItem.
  #[derive(Debug, Default)]
  pub struct FeedItem {
    pub title: RefCell<String>,
    pub url: RefCell<String>,
  }

  #[glib::object_subclass]
  impl ObjectSubclass for FeedItem {
    const NAME: &'static str = "FeedItem";
    type Type = super::FeedItem;
  }

  impl ObjectImpl for FeedItem {
    fn properties() -> &'static [glib::ParamSpec] {
      static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
        vec![
          glib::ParamSpecString::builder("title").build(),
          glib::ParamSpecString::builder("url").build(),
        ]
      });
      PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
      match pspec.name() {
        "title" => {
          self.title.replace(
            value
              .get()
              .expect("The value needs to be of type `String`."),
          );
        }
        "url" => {
          self.url.replace(
            value
              .get()
              .expect("The value needs to be of type `String`."),
          );
        }
        _ => unimplemented!(),
      }
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
      match pspec.name() {
        "title" => self.title.borrow().clone().to_value(),
        "url" => self.url.borrow().clone().to_value(),
        _ => unimplemented!(),
      }
    }
  }
}
