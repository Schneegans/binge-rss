// ------------------------------------------------------------------------------------ //
//                           ___ _               ___  ___ ___                           //
//                          | _ |_)_ _  __ _ ___| _ \/ __/ __|                          //
//                          | _ \ | ' \/ _` / -_)   /\__ \__ \                          //
//                          |___/_|_||_\__, \___|_|_\|___/___/                          //
//                                     |___/                                            //
// ------------------------------------------------------------------------------------ //

// SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
// SPDX-License-Identifier: MIT

use gtk::{gdk, gio, glib, prelude::*, subclass::prelude::*};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
  cell::{Ref, RefCell},
  error::Error,
  sync::atomic::{AtomicUsize, Ordering},
};

use crate::components::FeedItem;

// ---------------------------------------------------------------------------------------
#[derive(Debug, Copy, Clone, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "FeedState")]
pub enum FeedState {
  EmptyURL,
  DownloadPending,
  DownloadStarted,
  DownloadFailed,
  DownloadSucceeded,
}

impl Default for FeedState {
  fn default() -> Self {
    FeedState::EmptyURL
  }
}

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
    glib::Object::builder()
      .property("title", title)
      .property("url", url)
      .property("filter", filter)
      .property("viewed", viewed)
      .build()
  }

  // ---------------------------------------------------------------------- public methods

  pub fn download(&self) {
    if self.imp().download_source_id.borrow().is_some() {
      let source_id = self.imp().download_source_id.borrow_mut().take();
      source_id.unwrap().remove();
    }

    self.set_property("state", FeedState::DownloadStarted);

    let url_copy = self.imp().url.borrow().clone();

    let handle = crate::RUNTIME.spawn(async move {
      let bytes = reqwest::get(&url_copy).await?.bytes().await?;
      let content = feed_rs::parser::parse(&bytes[..])?;

      let url = url::Url::parse(&content.links[0].href);
      let icon_url = url.as_ref().unwrap().scheme().to_string()
        + &String::from("://")
        + &url.as_ref().unwrap().host().unwrap().to_string()
        + &String::from("/favicon.ico");

      let bytes = reqwest::get(icon_url).await?.bytes().await?;
      let image = Some(glib::Bytes::from(&bytes.to_vec()));

      Ok::<(feed_rs::model::Feed, Option<glib::Bytes>), Box<dyn Error + Send + Sync>>((
        content, image,
      ))
    });

    let ctx = glib::MainContext::default();
    self.imp().download_source_id.replace(
      Some(ctx.spawn_local(glib::clone!(@weak self as this => async move {

      let result = handle.await.unwrap();

      this.imp().download_source_id.replace(None);

      if result.is_ok() {

        let (content, image) = result.unwrap();

        let items = content.entries
        .iter()
        .map(|item| {
          let title = if item.title.is_some() {
            item.title.as_ref().unwrap().content.clone()
          } else {
            String::from("Unnamed Item")
          };

          let url = item.links[0].href.clone();

          FeedItem::new(title, url)
        })
        .collect();

        this.imp().items.replace(items);

        if image.is_some() {
          let stream = gio::MemoryInputStream::from_bytes(&image.unwrap());
          let pixbuf = gdk::gdk_pixbuf::Pixbuf::from_stream(&stream, gio::Cancellable::NONE);

          if pixbuf.is_ok() {
            let image = gtk::Image::from_pixbuf(Some(&pixbuf.unwrap()));
            this.imp().icon.replace(Some(image.paintable().unwrap()));
          }
        }

        this.set_property("state", FeedState::DownloadSucceeded);
      } else {
        this.set_property("state", FeedState::DownloadFailed);
      }
    }))));
  }

  pub fn get_title(&self) -> Ref<String> {
    self.imp().title.borrow()
  }

  pub fn get_url(&self) -> Ref<String> {
    self.imp().url.borrow()
  }

  pub fn get_filter(&self) -> Ref<String> {
    self.imp().filter.borrow()
  }

  pub fn get_viewed(&self) -> Ref<String> {
    self.imp().viewed.borrow()
  }

  pub fn get_id(&self) -> Ref<String> {
    self.imp().id.borrow()
  }

  pub fn get_state(&self) -> Ref<FeedState> {
    self.imp().state.borrow()
  }

  pub fn get_items(&self) -> Ref<Vec<FeedItem>> {
    self.imp().items.borrow()
  }

  pub fn get_icon(&self) -> Ref<Option<gdk::Paintable>> {
    self.imp().icon.borrow()
  }
}

mod imp {
  use super::*;

  // Object holding the state
  #[derive(Debug, Default)]
  pub struct Feed {
    pub title: RefCell<String>,
    pub url: RefCell<String>,
    pub filter: RefCell<String>,
    pub viewed: RefCell<String>,
    pub state: RefCell<FeedState>,

    pub id: RefCell<String>,
    pub items: RefCell<Vec<FeedItem>>,
    pub icon: RefCell<Option<gdk::Paintable>>,

    pub download_source_id: RefCell<Option<glib::SourceId>>,
  }

  // The central trait for subclassing a GObject
  #[glib::object_subclass]
  impl ObjectSubclass for Feed {
    const NAME: &'static str = "Feed";
    type Type = super::Feed;
  }

  // Trait shared by all GObjects
  impl ObjectImpl for Feed {
    fn constructed(&self) {
      self.parent_constructed();

      static COUNTER: AtomicUsize = AtomicUsize::new(0);
      self
        .id
        .replace(COUNTER.fetch_add(1, Ordering::Relaxed).to_string());
    }

    fn properties() -> &'static [glib::ParamSpec] {
      static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
        vec![
          glib::ParamSpecString::builder("title").build(),
          glib::ParamSpecString::builder("url").build(),
          glib::ParamSpecString::builder("filter").build(),
          glib::ParamSpecString::builder("viewed").build(),
          glib::ParamSpecEnum::builder::<FeedState>("state", FeedState::default())
            .build(),
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

          self.obj().imp().icon.replace(None);
          self.obj().imp().items.replace(vec![]);

          if self.url.borrow().is_empty() {
            self.obj().set_property("state", FeedState::EmptyURL);
          } else {
            self.obj().download();
          }
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
        "state" => {
          self.state.replace(
            value
              .get()
              .expect("The value needs to be of type `FeedState`."),
          );
        }
        _ => unimplemented!(),
      }
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
      match pspec.name() {
        "title" => self.title.borrow().clone().to_value(),
        "url" => self.url.borrow().clone().to_value(),
        "filter" => self.filter.borrow().clone().to_value(),
        "viewed" => self.viewed.borrow().clone().to_value(),
        "state" => self.state.borrow().clone().to_value(),
        _ => unimplemented!(),
      }
    }
  }
}
