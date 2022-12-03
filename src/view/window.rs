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
use gtk::{gio, glib, CompositeTemplate};

use crate::config;
use crate::model::Feed;
use crate::view::{FeedPage, FeedRow};

// ---------------------------------------------------------------------------------------
// The Window is derived from adw::Window. It primarily contains an adw::Leaflet with two
// panes: On the left, there is a sidebar with a list of all configured feeds, on the
// right there are details for the currently selected feed. The sidebar is realized as a
// gtk::ListBox full of custom FeedRows, the feed details page is a gtk::Stack containing
// a custom FeedPage for each feed.
glib::wrapper! {
  pub struct Window(ObjectSubclass<imp::Window>)
      @extends gtk::Widget, gtk::Window, adw::Window,
      @implements gtk::Accessible, gtk::Buildable;
}

impl Window {
  // ----------------------------------------------------------------- constructor methods

  pub fn new() -> Self {
    glib::Object::builder().build()
  }

  // ---------------------------------------------------------------------- public methods

  // This adds a new feed to the Window. The method will create a FeedRow for the list on
  // the left and a FeedPage for the details on the right. The ID of the feed is stored in
  // the widget names of the FeedRow and FeedPage. This allows us to later find the
  // widgets easily.
  pub fn add_feed(&self, feed: &Feed) {
    println!("add {}", feed.get_id());

    // If there are no feeds, the right pane of the main leaflet should not be visible. So
    // if a feed is added, we can make the leaflet unfoldable again.
    self.imp().leaflet.set_can_unfold(true);

    // Add a new FeedRow to the list on the left.
    let feed_row = FeedRow::new();
    feed_row.set_feed(feed);
    feed_row.set_widget_name(&feed.get_id());
    self.imp().feed_list.append(&feed_row);

    // Re-sort the FeedRows if the title of the Feed changed.
    feed.connect_notify_local(
      Some("title"),
      glib::clone!(@weak self as this => move |_, _| {
        this.imp().feed_list.invalidate_sort();
      }),
    );

    // Also update the title of the headerbar if the title of the Feed changed.
    feed
      .bind_property("title", &self.imp().header_label.get(), "label")
      .build();

    // Now add the FeedPage to show the Feed's detailed information.
    let feed_page = FeedPage::new();
    feed_page.set_feed(feed);
    self
      .imp()
      .feed_details
      .add_named(&feed_page, Some(&feed.get_id()));

    // Show the FeedPage if the FeedRow is activated.
    feed_row.connect_activated(
      glib::clone!(@weak self as this, @weak feed_page => move |feed_row| {
        this.show_feed_pages();
        this.imp().feed_details.set_visible_child(&feed_page);
        this.imp().header_label.set_label(&feed_row.title());
      }),
    );

    // Always select the last added feed.
    self.imp().feed_list.select_row(Some(&feed_row));
    self.imp().feed_details.set_visible_child(&feed_page);
    self.imp().header_label.set_label(&feed_row.title());

    // Make sure to update all widgets by emitting a state-change once.
    feed.notify("state");
  }

  // This method removes the currently selected feed from the user interface and returns
  // its ID. The next feed in the sidebar will become selected thereafter. If there is no
  // feed left, nothing will happen and the method will return None.
  pub fn remove_selected_feed(&self) -> Option<String> {
    let list = &self.imp().feed_list;
    let row = list.selected_row()?;
    let id = row.property::<String>("name");

    // Choose the item which will be selected after this operation. Usually, it will be
    // the feed below the currently selected feed. However, if the last feed is to be
    // deleted, we have to select the one above it.
    let mut next_row: Option<gtk::ListBoxRow> = None;

    // We have to check for the existence of the sibling's sibling to learn whether we are
    // the last row since there is also the no-items-placeholder child.
    if row.next_sibling().unwrap().next_sibling().is_some() {
      next_row = row
        .next_sibling()
        .unwrap()
        .downcast::<gtk::ListBoxRow>()
        .ok();
    } else if row.prev_sibling().is_some() {
      next_row = row
        .prev_sibling()
        .unwrap()
        .downcast::<gtk::ListBoxRow>()
        .ok();
    }

    // Remove the FeedRow from the sidebar.
    list.remove(&row);

    // Remove the FeedPage from the details stack.
    let page = self.get_feed_page(&id)?;
    self.imp().feed_details.remove(&page);

    // Clear the headerbar label.
    self.imp().header_label.set_label("");

    // If there is a next row to select, select it. Else show an info message about
    // creating the first feed.
    if next_row.is_some() {
      next_row.unwrap().activate();
    } else {
      self.imp().leaflet.set_can_unfold(false);
      self.show_feed_rows();
    }

    // Go to the sidebar pane if the leaflet is currently folded.
    if self.imp().leaflet.is_folded() {
      self.show_feed_rows();
    }

    Some(id)
  }

  // If the leaflet is folded, this will show the left sidebar area with the FeedRows.
  pub fn show_feed_rows(&self) {
    self
      .imp()
      .leaflet
      .set_visible_child(&self.imp().feed_list_page.get());
  }

  // If the leaflet is folded, this will show the right area with the FeedPages.
  pub fn show_feed_pages(&self) {
    self
      .imp()
      .leaflet
      .set_visible_child(&self.imp().feed_details_page.get());
  }

  // Shows a toast with the given message at the bottom of the screen.
  pub fn show_toast(
    &self,
    title: &str,
    button_label: &str,
    action_name: &str,
    action_target: &glib::Variant,
  ) {
    let toast = adw::Toast::builder()
      .title(title)
      .action_name(action_name)
      .action_target(action_target)
      .button_label(button_label)
      .build();
    self.imp().toast_overlay.add_toast(&toast);
  }

  // --------------------------------------------------------------------- private methods

  // Searches the gtk::Stack containing all FeedPages for the page corresponding to the
  // feed with the given ID. This will return None if no such page is found.
  fn get_feed_page(&self, id: &String) -> Option<FeedPage> {
    let page = self.imp().feed_details.child_by_name(id.as_str());

    if page.is_some() {
      Some(page?.downcast::<FeedPage>().unwrap())
    } else {
      None
    }
  }

  // Saves the current window size in the settings.
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

  // Restores the window size from the settings.
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

mod imp {
  use super::*;

  // -------------------------------------------------------------------------------------
  // The structure of the window widget is defined in the Window.ui file.
  #[derive(Debug, CompositeTemplate)]
  #[template(resource = "/io/github/schneegans/BingeRSS/ui/Window.ui")]
  pub struct Window {
    #[template_child]
    pub toast_overlay: TemplateChild<adw::ToastOverlay>,
    #[template_child]
    pub leaflet: TemplateChild<adw::Leaflet>,
    #[template_child]
    pub feed_list_page: TemplateChild<gtk::Box>,
    #[template_child]
    pub feed_details_page: TemplateChild<gtk::Box>,
    #[template_child]
    pub feed_list: TemplateChild<gtk::ListBox>,
    #[template_child]
    pub header_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub feed_details: TemplateChild<gtk::Stack>,
    pub settings: gio::Settings,
  }

  impl Default for Window {
    fn default() -> Self {
      Self {
        toast_overlay: TemplateChild::default(),
        leaflet: TemplateChild::default(),
        feed_list_page: TemplateChild::default(),
        feed_details_page: TemplateChild::default(),
        feed_list: TemplateChild::default(),
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

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
      obj.init_template();
    }
  }

  impl ObjectImpl for Window {
    // Most components of this custom widget are defined in the UI file. However, some
    // things have to be set up in code. This is done here, whenever a new Window is
    // constructed.
    fn constructed(&self) {
      self.parent_constructed();

      // Restore the window size from the previous session.
      self.obj().load_window_size();

      // Make sure that the FeedRows are sorted alphabetically.
      self.feed_list.set_sort_func(|a, b| -> gtk::Ordering {
        let a = a
          .downcast_ref::<adw::ActionRow>()
          .unwrap()
          .title()
          .to_lowercase();
        let b = b
          .downcast_ref::<adw::ActionRow>()
          .unwrap()
          .title()
          .to_lowercase();

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
    // Save the window size whenever the window is closed. We use this to be able to
    // restore the window size in the next session.
    fn close_request(&self) -> gtk::Inhibit {
      if let Err(err) = self.obj().save_window_size() {
        println!("Failed to save window state, {}", &err);
      }

      // Pass close request on to the parent
      self.parent_close_request()
    }
  }

  impl ApplicationWindowImpl for Window {}

  impl AdwApplicationWindowImpl for Window {}
}
