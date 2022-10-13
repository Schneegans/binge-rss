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

use crate::model::FeedItem;
use crate::view::FeedContentPage;

mod imp {
  use adw::subclass::prelude::AdwApplicationWindowImpl;

  use crate::config;

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
    pub header_label: TemplateChild<gtk::Label>,
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
        header_label: TemplateChild::default(),
        feed_details: TemplateChild::default(),
        settings: gio::Settings::new(config::APP_ID),
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

  pub fn get_selected_id(&self) -> Option<String> {
    match self.imp().feed_list.selected_row() {
      None => None,
      Some(row) => Some(row.widget_name().as_str().to_string()),
    }
  }

  pub fn get_selected_title(&self) -> Option<String> {
    match self.imp().feed_list.selected_row() {
      None => None,
      Some(row) => Some(
        row
          .downcast::<adw::ActionRow>()
          .unwrap()
          .title()
          .as_str()
          .to_string(),
      ),
    }
  }

  pub fn add_feed(&self, id: String, title: String, url: String) {
    let row = adw::ActionRow::builder()
      .activatable(true)
      .selectable(true)
      .name(&id)
      .title(&title)
      .build();
    self.imp().feed_list.append(&row);

    let spinner = gtk::Spinner::new();
    spinner.start();
    row.add_prefix(&spinner);

    let subpage = gtk::ScrolledWindow::builder().build();
    self.imp().feed_details.add_child(&subpage);

    if self.imp().feed_details.first_child() == self.imp().feed_details.last_child() {
      self.imp().feed_list.select_row(Some(&row));
    }

    let item_list = FeedContentPage::new();
    subpage.set_child(Some(&item_list));

    row.connect_activated(glib::clone!(@weak self as this => move |row| {
      this.show_details_page();
      this.imp().feed_details.set_visible_child(&subpage);
      this.imp().header_label.set_label(&row.title());
    }));

    let handle = crate::RUNTIME.spawn(async move {
      let bytes = reqwest::get(&url).await?.bytes().await?;
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
    ctx.spawn_local(glib::clone!(@weak self as this => async move {

      let feed = handle.await.unwrap();

      row.remove(&spinner);

      let avatar = adw::Avatar::builder().text(&title).size(24).icon_name("rss-symbolic").build();
      row.add_prefix(&avatar);

      if feed.is_ok() {

        let (content, image) = feed.unwrap();

        let items = content.entries
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

          FeedItem::new(title, url, summary)
        })
        .collect();

        if image.is_some() {
          let stream = gio::MemoryInputStream::from_bytes(&image.unwrap());
          let pixbuf = gdk::gdk_pixbuf::Pixbuf::from_stream(&stream, gio::Cancellable::NONE);

          if pixbuf.is_ok() {
            let image = gtk::Image::from_pixbuf(Some(&pixbuf.unwrap()));
            avatar.set_custom_image(Some(&image.paintable().unwrap()));
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

        item_list.set_items(items);

      } else {
        avatar.set_icon_name(Some("network-no-route-symbolic"));
        row.set_subtitle("Connection failed.");
        item_list.set_connection_failed();
      }
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
    let (width, height) = self.default_size();

    self.imp().settings.set_int("window-width", width)?;
    self.imp().settings.set_int("window-height", height)?;

    self
      .imp()
      .settings
      .set_boolean("is-maximized", self.is_maximized())?;

    Ok(())
  }

  fn load_window_size(&self) {
    let width = self.imp().settings.int("window-width");
    let height = self.imp().settings.int("window-height");
    let is_maximized = self.imp().settings.boolean("is-maximized");

    self.set_default_size(width, height);

    if is_maximized {
      self.maximize();
    }
  }
}
