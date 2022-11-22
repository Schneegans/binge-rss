// ------------------------------------------------------------------------------------ //
//                           ___ _               ___  ___ ___                           //
//                          | _ |_)_ _  __ _ ___| _ \/ __/ __|                          //
//                          | _ \ | ' \/ _` / -_)   /\__ \__ \                          //
//                          |___/_|_||_\__, \___|_|_\|___/___/                          //
//                                     |___/                                            //
// ------------------------------------------------------------------------------------ //

// SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
// SPDX-License-Identifier: MIT

use gtk::{glib, subclass::prelude::ObjectSubclassIsExt};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------------------
// The StoredFeed is used for storing the currently configured feeds in the settings.
// An array of such structs is converted from and to JSON using serde and stored under the
// GSettings key /io/github/schneegans/BingeRSS/feeds.
#[derive(Deserialize, Serialize, Debug)]
pub struct StoredFeed {
  // The user-defined name of the feed.
  pub title: String,

  // The URL to the feed xml.
  pub url: String,

  // The date at which the user last viewed the feed. This is used to compute the number
  // of new feed items.
  pub viewed: String,

  // The currently configured filter for this feed.
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub filter: String,
}

// ---------------------------------------------------------------------------------------
glib::wrapper! {
  pub struct Feed(ObjectSubclass<imp::Feed>);
}

impl Feed {
  // ----------------------------------------------------------------- constructor methods

  pub fn new(title: &String, url: &String, filter: &String, viewed: &String) -> Self {
    glib::Object::new(&[
      ("title", title),
      ("url", url),
      ("filter", filter),
      ("viewed", viewed),
    ])
    .expect("creating 'Feed'")
  }

  // ---------------------------------------------------------------------- public methods

  pub fn get_title(&self) -> String {
    self.imp().title.borrow().clone()
  }

  pub fn get_url(&self) -> String {
    self.imp().url.borrow().clone()
  }

  pub fn get_filter(&self) -> String {
    self.imp().filter.borrow().clone()
  }

  pub fn get_viewed(&self) -> String {
    self.imp().viewed.borrow().clone()
  }

  pub fn get_id(&self) -> String {
    self.imp().id.borrow().clone()
  }
  pub fn set_id(&self, id: String) {
    self.imp().id.replace(id);
  }
}

mod imp {
  use gtk::prelude::ToValue;
  use std::cell::RefCell;

  use glib::{ParamSpec, ParamSpecString, Value};
  use gtk::subclass::prelude::*;
  use once_cell::sync::Lazy;

  use super::*;

  // Object holding the state
  #[derive(Debug, Default)]
  pub struct Feed {
    pub title: RefCell<String>,
    pub url: RefCell<String>,
    pub filter: RefCell<String>,
    pub viewed: RefCell<String>,
    pub id: RefCell<String>,
  }

  // The central trait for subclassing a GObject
  #[glib::object_subclass]
  impl ObjectSubclass for Feed {
    const NAME: &'static str = "Feed";
    type Type = super::Feed;
  }

  // Trait shared by all GObjects
  impl ObjectImpl for Feed {
    fn properties() -> &'static [ParamSpec] {
      static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
        vec![
          ParamSpecString::builder("title").build(),
          ParamSpecString::builder("url").build(),
          ParamSpecString::builder("filter").build(),
          ParamSpecString::builder("viewed").build(),
          ParamSpecString::builder("id").build(),
        ]
      });
      PROPERTIES.as_ref()
    }

    fn set_property(
      &self,
      _obj: &Self::Type,
      _id: usize,
      value: &Value,
      pspec: &ParamSpec,
    ) {
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
        "filter" => {
          self.filter.replace(
            value
              .get()
              .expect("The value needs to be of type `String`."),
          );
        }
        "viewed" => {
          self.viewed.replace(
            value
              .get()
              .expect("The value needs to be of type `String`."),
          );
        }
        "id" => {
          self.id.replace(
            value
              .get()
              .expect("The value needs to be of type `String`."),
          );
        }
        _ => unimplemented!(),
      }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
      match pspec.name() {
        "title" => self.title.borrow().clone().to_value(),
        "url" => self.url.borrow().clone().to_value(),
        "filter" => self.filter.borrow().clone().to_value(),
        "viewed" => self.viewed.borrow().clone().to_value(),
        "id" => self.id.borrow().clone().to_value(),
        _ => unimplemented!(),
      }
    }
  }
}
