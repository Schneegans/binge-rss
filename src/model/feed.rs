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

use crate::model::FeedItem;

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

  // The unix timestamp in seconds at which the user last viewed the feed. This is used to
  // compute the number of unread feed items.
  pub viewed: i64,

  // The currently configured filter for this feed.
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub filter: String,
}

// ---------------------------------------------------------------------------------------
// Each Feed is in either of these states. Initially, the URL is empty. If the URL is set
// to any value, the state will change to DownloadPending. Then, if the download() method
// of the Feed is called, the state will change to DownloadStarted. If the download fails
// or succeeds, the state will change to either DownloadFailed or DownloadSucceeded. Since
// the state is a property of the Feed, you can get notified whenever it changes.
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
// Feed objects store the information on single feeds, like its name, url, or any applied
// filters. In addition, they allow to download the actual feed content from the internet.
// You can then access the individual feed items and an icon for the feed via its
// get_items() and get_icon() methods. There is also an interface for getting the number
// of unread items. This is done by comparing the publication timestamps of the feed items
// to the last time feed.set_viewed() was called. This may not work in all cases but it
// makes it unnecessary to store all feeds locally.
glib::wrapper! {
  pub struct Feed(ObjectSubclass<imp::Feed>);
}

impl Feed {
  // ----------------------------------------------------------------- constructor methods

  pub fn new(title: &String, url: &String, filter: &String, viewed: i64) -> Self {
    glib::Object::builder()
      .property("title", title)
      .property("url", url)
      .property("filter", filter)
      .property("viewed", viewed)
      .build()
  }

  // ---------------------------------------------------------------------- public methods

  // This method downloads the feed information from the configured URL. This happens in a
  // separate thread. If there is a download operation currently ongoing, it will be
  // canceled. As soon as the download starts, succeeds, or fails, the state property will
  // change accordingly. If the state changes to DownloadSucceeded, you can check the
  // items and icon of the Feed with the get_items() and get_icon() methods.
  pub fn download(&self) {
    // First cancel any ongoing download operation. This will not abort the actual
    // download thread, but we will ignore its result.
    if self.imp().download_source_id.borrow().is_some() {
      let source_id = self.imp().download_source_id.borrow_mut().take();
      source_id.unwrap().remove();
    }

    // Do nothing if the URL is empty.
    if self.get_state().eq(&FeedState::EmptyURL) {
      return;
    }

    // Notify about the started download operation.
    self.set_property("state", FeedState::DownloadStarted);

    let url_copy = self.imp().url.borrow().clone();

    // Spawn a thread for downloading the feed data.
    let handle = crate::RUNTIME.spawn(async move {
      // Download from the URL and parse the feed information.
      let bytes = reqwest::get(&url_copy).await?.bytes().await?;
      let content = feed_rs::parser::parse(&bytes[..])?;

      // Try to download the favicon from the feed.
      let url = url::Url::parse(&content.links[0].href);
      let icon_url = url.as_ref().unwrap().scheme().to_string()
        + &String::from("://")
        + &url.as_ref().unwrap().host().unwrap().to_string()
        + &String::from("/favicon.ico");

      let bytes = reqwest::get(icon_url).await?.bytes().await?;
      let image = Some(glib::Bytes::from(&bytes.to_vec()));

      // If everything succeed, return the feed's content and the bytes for the icon.
      Ok::<(feed_rs::model::Feed, Option<glib::Bytes>), Box<dyn Error + Send + Sync>>((
        content, image,
      ))
    });

    // Now spawn an asynchronous future on the main context. This will await the above
    // thread to finish and then store the feed items and the icon in our private members.
    // We will store the returned download_source_id so that we are able to cancel this if
    // download() is called again.
    let ctx = glib::MainContext::default();
    self.imp().download_source_id.replace(
      Some(ctx.spawn_local(glib::clone!(@weak self as this => async move {

      // Asynchronously wait for the above thread to finish. Once the thread finished, the
      // result will contain the feed content as well as the data for the icon.
      let result = handle.await.unwrap();

      // Reset the download_source_id.
      this.imp().download_source_id.replace(None);

      // Return early if the download failed.
      if result.is_err() {
        this.set_property("state", FeedState::DownloadFailed);
        return;
      }

      let (content, image) = result.unwrap();

      // Replace our title if it's still "New Feed".
      if content.title.is_some() && this.get_title().eq("New Feed") {
        this.set_property("title", content.title.unwrap().content.clone());
      }

      // Create FeedItems accordingly.
      this.imp().items.replace(content.entries
        .iter()
        .map(|item| {
          let title = if item.title.is_some() {
            item.title.as_ref().unwrap().content.clone()
          } else {
            String::from("Unnamed Item")
          };

          let url = item.links[0].href.clone();
          let date = if item.published.is_some() {item.published.unwrap().timestamp()} else {0};

          FeedItem::new(&title, &url, date)
        })
        .collect());

      // Convert the image data to a gdk::Paintable.
      if image.is_some() {
        let stream = gio::MemoryInputStream::from_bytes(&image.unwrap());
        let pixbuf = gdk::gdk_pixbuf::Pixbuf::from_stream(&stream, gio::Cancellable::NONE);

        if pixbuf.is_ok() {
          let image = gtk::Image::from_pixbuf(Some(&pixbuf.unwrap()));
          this.imp().icon.replace(Some(image.paintable().unwrap()));
        }
      }

      this.set_property("state", FeedState::DownloadSucceeded);

      // The number of unread items may have changed.
      this.notify("unread");
    }))));
  }

  // Return the name of the feed. If this is set to "New Feed", it will be overridden with
  // the actual feed title retrieved by the next call to download().
  pub fn get_title(&self) -> Ref<String> {
    self.imp().title.borrow()
  }

  // Get the configured URL.
  pub fn get_url(&self) -> Ref<String> {
    self.imp().url.borrow()
  }

  // Get the configured filter.
  pub fn get_filter(&self) -> Ref<String> {
    self.imp().filter.borrow()
  }

  // Get the automatically assigned unique ID for this feed. All constructed feeds will
  // have a different ID, however if the application is restarted, a feed may get a
  // different ID than last time.
  pub fn get_id(&self) -> Ref<String> {
    self.imp().id.borrow()
  }

  // Get the current state of the feed.
  pub fn get_state(&self) -> Ref<FeedState> {
    self.imp().state.borrow()
  }

  // Get all downloaded feed items. Initially, this will be empty.
  pub fn get_items(&self) -> Ref<Vec<FeedItem>> {
    self.imp().items.borrow()
  }

  // Get the downloaded icon for this feed. Initially and if something failed during the
  // download, this may be None.
  pub fn get_icon(&self) -> Ref<Option<gdk::Paintable>> {
    self.imp().icon.borrow()
  }

  // Get the last time the feed was viewed. That is, the last time set_viewed() was
  // called.
  pub fn get_viewed(&self) -> Ref<i64> {
    self.imp().viewed.borrow()
  }

  // Updates the 'viewed' property to contain the current time.
  pub fn set_viewed(&self) {
    self.set_property("viewed", chrono::Utc::now().timestamp());
    self.notify("unread");
  }

  // Returns the number of feed items which have been published after the last call to
  // set_viewed().
  pub fn get_unread(&self) -> i32 {
    self.property("unread")
  }
}

mod imp {
  use super::*;

  // -------------------------------------------------------------------------------------

  // Object holding the internal state of a Feed.
  #[derive(Debug, Default)]
  pub struct Feed {
    // This is set at construction time and unique amongst all feeds.
    pub id: RefCell<String>,

    // These are available as properties.
    pub title: RefCell<String>,
    pub url: RefCell<String>,
    pub filter: RefCell<String>,
    pub viewed: RefCell<i64>,
    pub state: RefCell<FeedState>,

    // These are set by the download() method.
    pub items: RefCell<Vec<FeedItem>>,
    pub icon: RefCell<Option<gdk::Paintable>>,
    pub download_source_id: RefCell<Option<glib::SourceId>>,
  }

  #[glib::object_subclass]
  impl ObjectSubclass for Feed {
    const NAME: &'static str = "Feed";
    type Type = super::Feed;
  }

  impl ObjectImpl for Feed {
    // Whenever a Feed is constructed, we generate a unique ID for it.
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
          glib::ParamSpecInt64::builder("viewed").build(),
          glib::ParamSpecInt::builder("unread").read_only().build(),
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
            self.obj().set_property("state", FeedState::DownloadPending);
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
          self
            .viewed
            .replace(value.get().expect("The value needs to be of type `i64`."));
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
        "unread" => (self
          .obj()
          .imp()
          .items
          .borrow()
          .iter()
          .filter(|i| i.is_newer(*self.obj().get_viewed()))
          .count() as i32)
          .to_value(),
        _ => unimplemented!(),
      }
    }
  }
}
