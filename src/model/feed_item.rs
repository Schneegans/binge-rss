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

  pub fn get_title(&self) -> String {
    self.imp().title.borrow().clone()
  }

  pub fn get_url(&self) -> String {
    self.imp().url.borrow().clone()
  }
}

mod imp {
  use super::*;

  // Object holding the state
  #[derive(Debug, Default)]
  pub struct FeedItem {
    pub title: RefCell<String>,
    pub url: RefCell<String>,
  }

  // The central trait for subclassing a GObject
  #[glib::object_subclass]
  impl ObjectSubclass for FeedItem {
    const NAME: &'static str = "FeedItem";
    type Type = super::FeedItem;
  }

  // Trait shared by all GObjects
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
