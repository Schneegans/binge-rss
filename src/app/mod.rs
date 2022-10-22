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
      window.set_icon_name(Some(config::APP_ID));

      if config::PROFILE == "develop" {
        window.add_css_class("devel");
      }

      window.connect_close_request(
        glib::clone!(@weak this => @default-return gtk::Inhibit(false), move |_| {
          this.save_data();
          gtk::Inhibit(false)
        }),
      );

      self.window.set(Some(&window));

      this.setup_actions();
      this.load_data();

      for (i, feed) in self.feeds.borrow().iter().enumerate() {
        window.add_feed(i.to_string(), feed.title.clone(), feed.url.clone());

        if feed.filter.len() > 0 {
          window.set_filter(&i.to_string(), &feed.filter[0]);
        }
      }

      window.select_first_feed();

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
        let dialog = adw::AboutWindow::builder()
          .application_name("BingeRSS")
          .license_type(gtk::License::MitX11)
          .application_icon(config::APP_ID)
          .developers(vec!["Simon Schneegans".into()])
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
      action.connect_activate(glib::clone!(@weak window => move |_, _| {
        let dialog = adw::MessageDialog::builder()
          .heading(&format!("Remove the feed '{}'?", window.get_selected_title().unwrap()))
          .default_response("remove")
          .close_response("cancel")
          .transient_for(&window)
          .modal(true)
          .build();
          dialog.add_response("cancel", "Cancel");
          dialog.add_response("remove", "Remove");
          dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);

          dialog.connect_response(Some("remove"), move |_,_| {
            let id = window.remove_selected_feed();

            println!("removed {:?}", id);
          });

        dialog.show();
      }));
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
