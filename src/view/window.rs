//////////////////////////////////////////////////////////////////////////////////////////
//                           ___ _               ___  ___ ___                           //
//                          | _ |_)_ _  __ _ ___| _ \/ __/ __|                          //
//                          | _ \ | ' \/ _` / -_)   /\__ \__ \                          //
//                          |___/_|_||_\__, \___|_|_\|___/___/                          //
//                                     |___/                                            //
//////////////////////////////////////////////////////////////////////////////////////////

// SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
// SPDX-License-Identifier: MIT

use std::error::Error;

use adw::prelude::*;
use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate};

use crate::model::{FeedData, FeedItemData};
use crate::view::FeedContentPage;

mod imp {
  use adw::subclass::prelude::AdwApplicationWindowImpl;

  use super::*;

  #[derive(Debug, CompositeTemplate)]
  #[template(resource = "/io/github/schneegans/BingeRSS/ui/Window.ui")]
  pub struct Window {
    #[template_child]
    pub leaflet: TemplateChild<adw::Leaflet>,
    #[template_child]
    pub feed_list: TemplateChild<gtk::ListBox>,
    #[template_child]
    pub add_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub feed_details: TemplateChild<gtk::Stack>,
    pub settings: gio::Settings,
  }

  impl Default for Window {
    fn default() -> Self {
      Self {
        leaflet: TemplateChild::default(),
        feed_list: TemplateChild::default(),
        add_button: TemplateChild::default(),
        feed_details: TemplateChild::default(),
        settings: gio::Settings::new("io.github.schneegans.BingeRSS"),
      }
    }
  }

  #[glib::object_subclass]
  impl ObjectSubclass for Window {
    const NAME: &'static str = "Window";
    type Type = super::Window;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
      klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
      obj.init_template();
    }
  }

  impl ObjectImpl for Window {
    fn constructed(&self, obj: &Self::Type) {
      self.parent_constructed(obj);

      obj.load_window_size();

      self.feed_list.set_sort_func(|a, b| -> gtk::Ordering {
        let a = a.downcast_ref::<adw::ActionRow>().unwrap().title();
        let b = b.downcast_ref::<adw::ActionRow>().unwrap().title();

        if a < b {
          gtk::Ordering::Smaller
        } else if a > b {
          gtk::Ordering::Larger
        } else {
          gtk::Ordering::Equal
        }
      });
    }
  }

  impl WidgetImpl for Window {}

  impl WindowImpl for Window {
    fn close_request(&self, window: &Self::Type) -> gtk::Inhibit {
      if let Err(err) = window.save_window_size() {
        println!("Failed to save window state, {}", &err);
      }

      // Pass close request on to the parent
      self.parent_close_request(window)
    }
  }

  impl ApplicationWindowImpl for Window {}

  impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, adw::Window, @implements gtk::Accessible, gtk::Buildable;
}

impl Window {
  pub fn new() -> Self {
    glib::Object::new(&[]).expect("Failed to create Window")
  }

  pub fn add_feed(&self, title: String, url: String) {
    let row = adw::ActionRow::builder()
      .activatable(true)
      .selectable(true)
      .sensitive(false)
      .title(title.as_str())
      .build();
    self.imp().feed_list.append(&row);

    let spinner = gtk::Spinner::new();
    spinner.start();
    row.add_prefix(&spinner);

    let handle = crate::RUNTIME.spawn(async move {
      let bytes = reqwest::get(&url).await?.bytes().await?;
      let content = feed_rs::parser::parse(&bytes[..])?;

      let mut data = FeedData {
        items: vec![],
        image: None,
      };

      data.items = content
        .entries
        .iter()
        .map(|item| {
          let title = if item.title.is_some() {
            item.title.as_ref().unwrap().content.clone()
          } else {
            String::from("Unnamed Item")
          };

          let url = item.links[0].href.clone();

          let summary = if item.summary.is_some() {
            item.summary.as_ref().unwrap().content.clone()
          } else {
            String::from("")
          };

          FeedItemData {
            title: title,
            url: url,
            summary: summary,
          }
        })
        .collect();

      let url = url::Url::parse(&content.links[0].href);
      let icon_url = url.as_ref().unwrap().scheme().to_string()
        + &String::from("://")
        + &url.as_ref().unwrap().host().unwrap().to_string()
        + &String::from("/favicon.ico");

      let bytes = reqwest::get(icon_url).await?.bytes().await?;
      data.image = Some(glib::Bytes::from(&bytes.to_vec()));

      Ok::<FeedData, Box<dyn Error + Send + Sync>>(data)
    });

    let ctx = glib::MainContext::default();
    ctx.spawn_local(glib::clone!(@weak self as this => async move {
      let feed = handle.await.unwrap().unwrap();

      row.remove(&spinner);
      row.set_sensitive(true);

      if feed.image.is_some() {
        let stream = gio::MemoryInputStream::from_bytes(&feed.image.unwrap());
        let pixbuf = gdk::gdk_pixbuf::Pixbuf::from_stream(&stream, gio::Cancellable::NONE);

        if pixbuf.is_ok() {
          let image = gtk::Image::from_pixbuf(Some(&pixbuf.unwrap()));
          row.add_prefix(&image);
        }
      }

      {
        let unread_count = 43;
        let label = gtk::Label::builder()
          .label(&unread_count.to_string())
          .valign(gtk::Align::Center)
          .css_classes(vec!["item-count-badge".to_string()])
          .build();

        row.add_suffix(&label);
      }

      let subpage = gtk::ScrolledWindow::builder().build();

      let item_list = FeedContentPage::new();
      item_list.set_items(feed.items);

      subpage.set_child(Some(&item_list));

      this.imp().feed_details.add_child(&subpage);

      if this.imp().feed_details.first_child() == this.imp().feed_details.last_child() {
        this.imp().feed_list.select_row(Some(&row));
      }

      row.connect_activated( move |_| {
        this.show_details_page();
        this.imp().feed_details.set_visible_child(&subpage);
      });
    }));
  }

  pub fn show_feed_page(&self) {
    self.imp().leaflet.set_visible_child_name("feed_list_page");
  }

  pub fn show_details_page(&self) {
    self
      .imp()
      .leaflet
      .set_visible_child_name("feed_details_page");
  }

  fn save_window_size(&self) -> Result<(), glib::BoolError> {
    let imp = self.imp();

    let (width, height) = self.default_size();

    imp.settings.set_int("window-width", width)?;
    imp.settings.set_int("window-height", height)?;

    imp
      .settings
      .set_boolean("is-maximized", self.is_maximized())?;

    Ok(())
  }

  fn load_window_size(&self) {
    let imp = self.imp();

    let width = imp.settings.int("window-width");
    let height = imp.settings.int("window-height");
    let is_maximized = imp.settings.boolean("is-maximized");

    self.set_default_size(width, height);

    if is_maximized {
      self.maximize();
    }
  }
}
