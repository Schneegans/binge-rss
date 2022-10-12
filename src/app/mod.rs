//////////////////////////////////////////////////////////////////////////////////////////
//                           ___ _               ___  ___ ___                           //
//                          | _ |_)_ _  __ _ ___| _ \/ __/ __|                          //
//                          | _ \ | ' \/ _` / -_)   /\__ \__ \                          //
//                          |___/_|_||_\__, \___|_|_\|___/___/                          //
//                                     |___/                                            //
//////////////////////////////////////////////////////////////////////////////////////////

// SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
// SPDX-License-Identifier: MIT

use adw::prelude::*;
use glib::WeakRef;
use gtk::{gio, glib, subclass::prelude::*};
use std::cell::RefCell;

use crate::config;
use crate::model::FeedSettings;
use crate::view::Window;

mod imp {

  use adw::subclass::prelude::AdwApplicationImpl;

  use super::*;

  #[derive(Debug)]
  pub struct Application {
    pub window: WeakRef<Window>,
    pub settings: gio::Settings,
    pub feeds: RefCell<Vec<FeedSettings>>,
  }

  impl Default for Application {
    fn default() -> Self {
      Self {
        window: Default::default(),
        settings: gio::Settings::new(config::APP_ID),
        feeds: RefCell::new(vec![]),
      }
    }
  }

  #[glib::object_subclass]
  impl ObjectSubclass for Application {
    const NAME: &'static str = "Application";
    type Type = super::Application;
    type ParentType = adw::Application;
  }

  impl ObjectImpl for Application {}

  impl ApplicationImpl for Application {
    fn activate(&self, this: &Self::Type) {
      if let Some(window) = self.window.upgrade() {
        window.show();
        window.present();
        return;
      }

      let window = Window::new();
      window.set_application(Some(this));
      window.set_title(Some(&"BingeRSS".to_string()));

      window.connect_close_request(
        glib::clone!(@weak this => @default-return gtk::Inhibit(false), move |_| {
          this.save_data();
          gtk::Inhibit(false)
        }),
      );

      self.window.set(Some(&window));

      this.setup_actions();
      this.load_data();

      for feed in self.feeds.borrow().iter() {
        window.add_feed(feed.title.clone(), feed.url.clone());
      }

      this.main_window().present();
    }
  }

  impl GtkApplicationImpl for Application {}
  impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application, @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
  pub fn new() -> Self {
    glib::Object::new(&[("application-id", &Some(config::APP_ID))])
      .expect("Application initialization failed")
  }

  fn main_window(&self) -> Window {
    self.imp().window.upgrade().unwrap()
  }

  fn setup_actions(&self) {
    let window = self.main_window();
    {
      let action = gio::SimpleAction::new("about", None);
      action.connect_activate(glib::clone!(@weak window => move |_, _| {
        let dialog = gtk::AboutDialog::builder()
          .program_name("BingeRSS")
          .license_type(gtk::License::MitX11)
          .logo_icon_name(config::APP_ID)
          .authors(vec!["Simon Schneegans".into()])
          .artists(vec!["Simon Schneegans".into()])
          .website("https://github.com/schneegans/binge-rss")
          .version(config::VERSION)
          .transient_for(&window)
          .modal(true)
          .build();

        dialog.present();
      }));

      self.add_action(&action);
    }

    {
      let action = gio::SimpleAction::new("quit", None);
      action.connect_activate(glib::clone!(@weak window => move |_, _| {
        window.close();
      }));

      self.add_action(&action);
    }

    {
      let action = gio::SimpleAction::new("add-feed", None);
      action.connect_activate(move |_, _| {
        println!("add");
      });
      self.add_action(&action);
    }

    {
      let action = gio::SimpleAction::new("remove-feed", None);
      action.connect_activate(move |_, _| {
        println!("remove");
      });
      self.add_action(&action);
    }

    {
      let action = gio::SimpleAction::new("show-feed-page", None);
      action.connect_activate(glib::clone!(@weak window => move |_, _| {
        window.show_feed_page();
      }));
      self.add_action(&action);
    }
  }

  fn load_data(&self) {
    let data = self.imp().settings.string("feeds");
    self
      .imp()
      .feeds
      .replace(serde_json::from_str(data.as_str()).expect("valid json"));
  }

  fn save_data(&self) {
    let json = serde_json::to_string(&self.imp().feeds).unwrap();
    self
      .imp()
      .settings
      .set_string("feeds", &json)
      .expect("Failed to write settings!");
  }
}

impl Default for Application {
  fn default() -> Self {
    gio::Application::default()
      .unwrap()
      .downcast::<Application>()
      .unwrap()
  }
}
