// ------------------------------------------------------------------------------------ //
//                           ___ _               ___  ___ ___                           //
//                          | _ |_)_ _  __ _ ___| _ \/ __/ __|                          //
//                          | _ \ | ' \/ _` / -_)   /\__ \__ \                          //
//                          |___/_|_||_\__, \___|_|_\|___/___/                          //
//                                     |___/                                            //
// ------------------------------------------------------------------------------------ //

// SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
// SPDX-License-Identifier: MIT

use adw::prelude::*;
use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};

use crate::components::Feed;
use crate::components::FeedContentPage;
use crate::components::FeedRow;

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
          .into(),
      ),
    }
  }

  pub fn add_feed(&self, feed: &Feed) {
    println!("add {}", feed.get_id());
    self.imp().no_feeds_message.set_visible(false);
    self.imp().leaflet.set_can_unfold(true);

    let feed_row = FeedRow::new();
    feed_row.set_feed(feed);
    feed_row.set_widget_name(&feed.get_id());
    self.imp().feed_list.append(&feed_row);

    feed
      .bind_property("title", &self.imp().header_label.get(), "label")
      .build();

    feed.connect_notify_local(
      Some("title"),
      glib::clone!(@weak self as this => move |_, _| {
        this.imp().feed_list.invalidate_sort();
      }),
    );

    feed.connect_notify_local(
      Some("url"),
      glib::clone!(@weak feed => move |_, _| {
        feed.download();
      }),
    );

    let feed_page = FeedContentPage::new();
    feed_page.set_feed(feed);
    self
      .imp()
      .feed_details
      .add_named(&feed_page, Some(&feed.get_id()));

    feed_row.connect_activated(
      glib::clone!(@weak self as this, @weak feed_page => move |feed_row| {
        this.show_details_page();
        this.imp().feed_details.set_visible_child(&feed_page);
        this.imp().header_label.set_label(&feed_row.title());
      }),
    );

    self.imp().feed_list.select_row(Some(&feed_row));
    self.imp().feed_details.set_visible_child(&feed_page);
    self.imp().header_label.set_label(&feed_row.title());

    feed.connect_local(
      "download-started",
      false,
      glib::clone!(@weak feed_row => @default-return None, move |_| {
        feed_row.imp().spinner.set_visible(true);
        feed_row.imp().avatar.set_visible(false);

        None
      }),
    );

    feed.connect_local(
      "download-finished",
      false,
      glib::clone!(@weak feed, @weak feed_row, @weak feed_page => @default-return None, move |success| {
        let success = success[0].get::<bool>().unwrap();

        feed_row.imp().spinner.set_visible(!success);
        feed_row.imp().avatar.set_visible(success);
        feed_row.set_connection_failed(!success);
        feed_row.imp().badge.set_visible(success);

        feed_row
          .imp()
          .avatar
          .set_custom_image(feed.get_icon().as_ref());
        feed_page.set_items(feed.get_items().as_ref());

        if !success {
          feed_page.set_connection_failed();
        }

        None
      }),
    );

    feed.download();

    self.imp().feed_list.invalidate_sort();
  }

  pub fn show_toast(
    &self,
    title: &str,
    button_label: &str,
    action_name: &str,
    action_target: glib::Variant,
  ) {
    let toast = adw::Toast::builder()
      .title(title)
      .action_name(action_name)
      .action_target(&action_target)
      .button_label(button_label)
      .build();
    self.imp().toast_overlay.add_toast(&toast);
  }

  pub fn remove_selected_feed(&self) -> Option<String> {
    let list = &self.imp().feed_list;
    let row = list.selected_row()?;
    let id = row.property::<String>("name");

    let mut next_row: Option<gtk::ListBoxRow> = None;

    if row.next_sibling().is_some() {
      next_row = Some(
        row
          .next_sibling()
          .unwrap()
          .downcast::<gtk::ListBoxRow>()
          .unwrap(),
      );
    } else if row.prev_sibling().is_some() {
      next_row = Some(
        row
          .prev_sibling()
          .unwrap()
          .downcast::<gtk::ListBoxRow>()
          .unwrap(),
      );
    }

    list.remove(&row);

    let page = self.get_feed_content_page(&id)?;
    self.imp().feed_details.remove(&page);

    self.imp().header_label.set_label("");

    if next_row.is_some() {
      next_row.unwrap().activate();
    } else {
      self.imp().no_feeds_message.set_visible(true);
      self.imp().leaflet.set_can_unfold(false);
      self.show_feed_page();
    }

    if self.imp().leaflet.is_folded() {
      self.show_feed_page();
    }

    Some(id)
  }

  pub fn show_feed_page(&self) {
    self
      .imp()
      .leaflet
      .set_visible_child(&self.imp().feed_list_page.get());
  }

  pub fn show_details_page(&self) {
    self
      .imp()
      .leaflet
      .set_visible_child(&self.imp().feed_details_page.get());
  }

  // --------------------------------------------------------------------- private methods

  fn get_feed_content_page(&self, id: &String) -> Option<FeedContentPage> {
    let page = self.imp().feed_details.child_by_name(id.as_str());

    if page.is_some() {
      Some(page?.downcast::<FeedContentPage>().unwrap())
    } else {
      None
    }
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

mod imp {
  use adw::subclass::prelude::AdwApplicationWindowImpl;

  use crate::config;

  use super::*;

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
    #[template_child]
    pub no_feeds_message: TemplateChild<adw::StatusPage>,
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
        no_feeds_message: TemplateChild::default(),
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
    fn constructed(&self) {
      self.parent_constructed();

      self.obj().load_window_size();

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
