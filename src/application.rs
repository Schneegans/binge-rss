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

  // This creates all the actions which glue together all the parts of BingeRSS. There are
  // currently these actions available:
  //   app.about():          Shows the about dialog.
  //   app.quit():           Quits the application.
  //   app.add-feed():       Adds a new empty feed.
  //   app.remove-feed():    Removes the currently selected feed and shows a undo-toast.
  //   app.show-feed-page(): If folded, this shows the pane of the main leaflet.
  //   app.undo-remove(id):  Re-adds a previously deleted feed. The ID of the
  //                         to-be-re-added feed has to be given as parameter.
  //   app.refresh():        Re-downloads all feeds.
  fn setup_actions(&self) {
    let window = self.main_window();

    // Show an about dialog if app.about() is called.
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

    // Quit BingeRSS if app.quit() is called.
    {
      let action = gio::SimpleAction::new("quit", None);
      action.connect_activate(glib::clone!(@weak window => move |_, _| {
        window.close();
      }));

      self.add_action(&action);
    }

    // Add a new empty feed if app.add-feed() is called.
    {
      let action = gio::SimpleAction::new("add-feed", None);
      action.connect_activate(
        glib::clone!(@weak self as this, @weak window => move |_, _| {

          // First, create the new feed.
          let feed = Feed::new(&"New Feed".into(), &"".into(), &"".into(), &"".into());

          // Then add it to the user interface.
          window.add_feed(&feed);

          // Finally, store it in the list of all feeds.
          this.imp().feeds.borrow_mut().push(feed);
        }),
      );
      self.add_action(&action);
    }

    // Remove the currently selected feed if app.remove-feed() is called. This will show a
    // toast which will allow to undo this destructive action for a couple of seconds.
    {
      let action = gio::SimpleAction::new("remove-feed", None);
      action.connect_activate(
        glib::clone!(@weak self as this, @weak window => move |_, _| {

          // First, remove the currently selected feed from the user interface. This will
          // return the ID of the removed feed (if any).
          let id = window.remove_selected_feed();
          if id.is_none() {
            return;
          }

          // Get the index of the removed feed in the list of all feeds.
          let i = this.imp().feeds.borrow().iter().position(|f| {
            f.get_id().eq(id.as_ref().unwrap())
          }).unwrap();

          // Show the undo-toast. It has a button which will call the app.undo-remove()
          // action and pass the ID of the removed feed to the action.
          window.show_toast(
            format!("Removed '{}'", this.imp().feeds.borrow()[i].get_title()).as_str(),
            "Undo",
            "app.undo-remove",
            id.unwrap().to_variant());

          // Remove the feed from the list of all feeds and add it to the list of all
          // removed feeds instead. This will allow us to later undo the deletion of the
          // feed.
          let feed = this.imp().feeds.borrow_mut().remove(i);
          this.imp().removed_feeds.borrow_mut().push(feed);
        }),
      );
      self.add_action(&action);
    }

    // Add the app.undo-remove(id) action, which is called when the user clicks the 'Undo'
    // button in the toast which is shown whenever a feed is deleted.
    {
      let action = gio::SimpleAction::new("undo-remove", Some(glib::VariantTy::STRING));
      action.connect_activate(
        glib::clone!(@weak self as this, @weak window => move |_, id| {
            if id.is_some() {

              // First, get the index of the to-be-restored feed from the list of all
              // previously removed feeds.
              let i = this.imp().removed_feeds.borrow().iter().position(|f| {
                f.get_id().eq(&String::from_variant(id.unwrap()).unwrap())
              }).unwrap();

              // Then remove the feed from the list of removed feeds, re-add it to the
              // user interface and store it in the real feed list.
              let feed = this.imp().removed_feeds.borrow_mut().remove(i);
              window.add_feed(&feed);
              this.imp().feeds.borrow_mut().push(feed);
            }
        }),
      );
      self.add_action(&action);
    }

    // If the main leaflet is folded, this will show the left pane. This is used for the
    // back-navigation button in the headerbar.
    {
      let action = gio::SimpleAction::new("show-feed-page", None);
      action.connect_activate(glib::clone!(@weak window => move |_, _| {
        window.show_feed_page();
      }));
      self.add_action(&action);
    }

    // The app.refresh() action simply downloads all configured feeds.
    {
      let action = gio::SimpleAction::new("refresh", None);
      action.connect_activate(glib::clone!(@weak self as this => move |_, _| {
        this.imp().feeds.borrow().iter().for_each(|f| f.download());
      }));
      self.add_action(&action);
    }
  }

  // The feeds are stored in a JSON string under the settings key
  // io.github.schneegans.bingerss.feeds. This method retrieves the JSON string, and
  // creates Feed objects accordingly. The newly created Feed objects are added to the
  // user interface.
  fn load_feeds(&self) {
    // Load the JSON string.
    let data = self.imp().settings.string("feeds");

    // Parse it.
    let stored_feeds: Vec<StoredFeed> =
      serde_json::from_str(data.as_str()).expect("valid json");

    // Create a Feed for each StoredFeed.
    self.imp().feeds.replace(
      stored_feeds
        .iter()
        .map(|f| Feed::new(&f.title, &f.url, &f.filter, &f.viewed))
        .collect(),
    );

    // Add all feeds to the user interface.
    for feed in self.imp().feeds.borrow_mut().iter_mut() {
      self.main_window().add_feed(&feed);
    }
  }

  // The feeds are stored in a JSON string under the settings key
  // io.github.schneegans.bingerss.feeds. This method converts all current Feeds to a JSON
  // string, and saves this data under the settings key.
  fn save_feeds(&self) {
    // Create a StoredFeed for each Feed.
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

    // Serialize the data to JSON.
    let json = serde_json::to_string(&stored_feeds).unwrap();

    // Write the JSON string to the settings.
    self
      .imp()
      .settings
      .set_string("feeds", &json)
      .expect("Failed to write settings!");
  }

  // Returns the current main window of the application. This will panic if called before
  // the initial call to activate().
  fn main_window(&self) -> Window {
    self.imp().window.upgrade().unwrap()
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
    // This is called when the application is started and for each subsequent attempt of
    // the user to start another instance of the application. In the latter cases, no new
    // application instance is opened, instead this method is called on the primary
    // instance.
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
          this.obj().save_feeds();
          gtk::Inhibit(false)
        }),
      );

      // Setup the actions which glue to together the functionality of BingeRSS.
      self.obj().setup_actions();

      // Load all configured feeds from the settings.
      self.obj().load_feeds();

      // Finally, show the window.
      self.obj().main_window().present();
    }
  }

  impl GtkApplicationImpl for Application {}
  impl AdwApplicationImpl for Application {}
}
