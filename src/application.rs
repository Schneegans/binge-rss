// ------------------------------------------------------------------------------------ //
//                           ___ _               ___  ___ ___                           //
//                          | _ |_)_ _  __ _ ___| _ \/ __/ __|                          //
//                          | _ \ | ' \/ _` / -_)   /\__ \__ \                          //
//                          |___/_|_||_\__, \___|_|_\|___/___/                          //
//                                     |___/                                            //
// ------------------------------------------------------------------------------------ //

// SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
// SPDX-License-Identifier: MIT

use adw::{prelude::*, subclass::prelude::*};
use gtk::{gio, glib, glib::FromVariant, glib::WeakRef};
use std::cell::RefCell;

use crate::config;
use crate::model::Feed;
use crate::model::StoredFeed;
use crate::view::Window;

// ---------------------------------------------------------------------------------------
// The application of BingeRSS is derived from adw::Application. It does not have any
// additional public methods; all the setup happens in the overridden activate() methods.
glib::wrapper! {
  pub struct Application(ObjectSubclass<imp::Application>)
    @extends gio::Application, gtk::Application, adw::Application,
    @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
  // ----------------------------------------------------------------- constructor methods

  // Creates a new instance of the application class.
  pub fn new() -> Self {
    glib::Object::builder()
      .property("application-id", &Some(config::APP_ID))
      .build()
  }

  // --------------------------------------------------------------------- private methods

  // Returns the current main window of the application.
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
      action.connect_activate(
        glib::clone!(@weak self as this, @weak window => move |_, _| {
          let feed = Feed::new(
             &"New Feed".into(),
             &"".into(),
             &"".into(),
             &"".into()
          );

          window.add_feed(&feed);
          this.imp().feeds.borrow_mut().push(feed);
        }),
      );
      self.add_action(&action);
    }

    {
      let action = gio::SimpleAction::new("remove-feed", None);
      action.connect_activate(
        glib::clone!(@weak self as this, @weak window => move |_, _| {
          let id = window.remove_selected_feed();

          if id.is_some() {
            let i = this.imp().feeds.borrow().iter().position(|f| f.get_id().eq(id.as_ref().unwrap())).unwrap();
            window.show_toast(format!("Removed '{}'", this.imp().feeds.borrow()[i].get_title()).as_str(), "Undo", "app.undo-remove", id.unwrap().to_variant());
            this.imp().removed_feeds.borrow_mut().push(this.imp().feeds.borrow_mut().remove(i));
          }
        }),
      );
      self.add_action(&action);
    }

    {
      let action = gio::SimpleAction::new("undo-remove", Some(glib::VariantTy::STRING));
      action.connect_activate(
        glib::clone!(@weak self as this, @weak window => move |_, id| {
            if id.is_some() {
              let i = this.imp().removed_feeds.borrow().iter().position(|f| f.get_id().eq(&String::from_variant(id.unwrap()).unwrap())).unwrap();
              let feed = this.imp().removed_feeds.borrow_mut().remove(i);
              window.add_feed(&feed);
              this.imp().feeds.borrow_mut().push(feed);
            }
        }),
      );
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
    let stored_feeds: Vec<StoredFeed> =
      serde_json::from_str(data.as_str()).expect("valid json");

    self.imp().feeds.replace(
      stored_feeds
        .iter()
        .map(|f| Feed::new(&f.title, &f.url, &f.filter, &f.viewed))
        .collect(),
    );

    for feed in self.imp().feeds.borrow_mut().iter_mut() {
      self.main_window().add_feed(&feed);
    }
  }

  fn save_data(&self) {
    let stored_feeds: Vec<StoredFeed> = self
      .imp()
      .feeds
      .borrow()
      .iter()
      .map(|f| StoredFeed {
        title: f.get_title().clone(),
        url: f.get_url().clone(),
        filter: f.get_filter().clone(),
        viewed: f.get_viewed().clone(),
      })
      .collect();

    let json = serde_json::to_string(&stored_feeds).unwrap();
    self
      .imp()
      .settings
      .set_string("feeds", &json)
      .expect("Failed to write settings!");
  }
}

mod imp {
  use super::*;

  // -------------------------------------------------------------------------------------
  // This object holds the state of our custom application. Next to the current
  // application window and the GSettings, it contains a list of all currently configured
  // feeds. If a feed gets removed by the user, it is removed from the 'feeds' but added
  // to the 'removed_feeds'. This allows us to undo the deletion if required.
  #[derive(Debug)]
  pub struct Application {
    pub window: WeakRef<Window>,
    pub settings: gio::Settings,
    pub feeds: RefCell<Vec<Feed>>,
    pub removed_feeds: RefCell<Vec<Feed>>,
  }

  impl Default for Application {
    fn default() -> Self {
      Self {
        window: Default::default(),
        settings: gio::Settings::new(config::APP_ID),
        feeds: RefCell::new(vec![]),
        removed_feeds: RefCell::new(vec![]),
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
    fn activate(&self) {
      // If the app is already running and we created a window before, simply show it.
      if let Some(window) = self.window.upgrade() {
        window.show();
        window.present();
        return;
      }

      // Else, the app was not running and we have to create a new window.
      let window = Window::new();
      window.set_application(Some(self.obj().as_ref()));
      window.set_title(Some(&"BingeRSS".to_string()));
      window.set_icon_name(Some(config::APP_ID));
      self.window.set(Some(&window));

      // Add the cool striped headerbar if we compile a development version.
      if config::PROFILE == "develop" {
        window.add_css_class("devel");
      }

      // Save the current feeds whenever the window gets closed.
      window.connect_close_request(
        glib::clone!(@weak self as this => @default-return gtk::Inhibit(false), move |_| {
          this.obj().save_data();
          gtk::Inhibit(false)
        }),
      );

      // Setup the actions which glue to together the functionality of BingeRSS.
      self.obj().setup_actions();

      // Load all configured feeds from the settings.
      self.obj().load_data();

      // Finally, show the window.
      self.obj().main_window().present();
    }
  }

  impl GtkApplicationImpl for Application {}
  impl AdwApplicationImpl for Application {}
}
