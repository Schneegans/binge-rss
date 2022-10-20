use gtk::{glib, subclass::prelude::ObjectSubclassIsExt};

mod imp {
  use gtk::prelude::ToValue;
  use std::cell::RefCell;

  use glib::{ParamSpec, ParamSpecString, Value};
  use gtk::subclass::prelude::*;
  use once_cell::sync::Lazy;

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
    fn properties() -> &'static [ParamSpec] {
      static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
        vec![
          ParamSpecString::builder("title").build(),
          ParamSpecString::builder("url").build(),
        ]
      });
      PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
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

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
      match pspec.name() {
        "title" => self.title.borrow().clone().to_value(),
        "url" => self.url.borrow().clone().to_value(),
        _ => unimplemented!(),
      }
    }
  }
}

glib::wrapper! {
    pub struct FeedItem(ObjectSubclass<imp::FeedItem>);
}

impl FeedItem {
  pub fn new(title: String, url: String) -> Self {
    glib::Object::new(&[("title", &title), ("url", &url)]).expect("creating 'FeedItem'")
  }

  pub fn get_title(&self) -> String {
    self.imp().title.borrow().clone()
  }

  pub fn get_url(&self) -> String {
    self.imp().url.borrow().clone()
  }
}
