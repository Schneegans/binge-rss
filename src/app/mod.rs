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
use gio::Settings;
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
    pub settings: Settings,
    pub feeds: RefCell<Vec<FeedSettings>>,
  }

  impl Default for Application {
    fn default() -> Self {
      Self {
        window: Default::default(),
        settings: Settings::new(config::APP_ID),
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
    fn activate(&self, app: &Self::Type) {
      if let Some(window) = self.window.upgrade() {
        window.show();
        window.present();
        return;
      }

      let window = Window::new();
      window.set_application(Some(app));
      window.set_title(Some(&"BingeRSS".to_string()));

      self.window.set(Some(&window));

      app.setup_actions();

      let data = r#"
        [
          {
            "title": "Der SPIEGEL",
            "url": "https://www.spiegel.de/schlagzeilen/tops/index.rss",
            "viewed": "2022-10-09 16:06:14 UTC"
          },
          {
            "title": "Unixporn",
            "url": "http://reddit.com/r/unixporn/new/.rss?sort=new",
            "viewed": "2022-10-09 16:06:14 UTC"
          },
          {
            "title": "Forschung Aktuell",
            "url": "https://www.deutschlandfunk.de/forschung-aktuell-104.xml",
            "viewed": "2022-10-09 16:06:14 UTC"
          },
          {
            "title": "Linux",
            "url": "http://reddit.com/r/linux/new/.rss?sort=new",
            "viewed": "2022-10-09 16:06:14 UTC"
          },
          {
            "title": "GNOME",
            "url": "http://reddit.com/r/gnome/new/.rss?sort=new",
            "viewed": "2022-10-09 16:06:14 UTC"
          },
          {
            "title": "OMG Ubuntu",
            "url": "https://omgubuntu.co.uk/feed",
            "viewed": "2022-10-09 16:06:14 UTC"
          },
          {
            "title": "Blendernation",
            "url": "https://www.blendernation.com/feed/",
            "viewed": "2022-10-09 16:06:14 UTC"
          },
          {
            "title": "The Verge",
            "url": "https://www.theverge.com/rss/index.xml",
            "viewed": "2022-10-09 16:06:14 UTC"
          },
          {
            "title": "Ars Technica",
            "url": "https://feeds.arstechnica.com/arstechnica/features",
            "viewed": "2022-10-09 16:06:14 UTC"
          },
          {
            "title": "Hacker News",
            "url": "https://news.ycombinator.com/rss",
            "viewed": "2022-10-09 16:06:14 UTC"
          },
          {
            "title": "Vulnerabilities",
            "url": "https://nvd.nist.gov/feeds/xml/cve/misc/nvd-rss-analyzed.xml",
            "viewed": "2022-10-09 16:06:14 UTC"
          }
        ]"#;

      self
        .feeds
        .replace(serde_json::from_str(data).expect("valid json"));

      for feed in self.feeds.borrow().iter() {
        window.add_feed(feed.title.clone(), feed.url.clone());
      }

      app.main_window().present();
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
}

impl Default for Application {
  fn default() -> Self {
    gio::Application::default()
      .unwrap()
      .downcast::<Application>()
      .unwrap()
  }
}
