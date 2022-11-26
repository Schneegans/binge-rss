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
use gtk::{gdk, gio, glib, pango, subclass::prelude::*, CompositeTemplate};

use crate::components::Feed;
use crate::components::FeedItem;

glib::wrapper! {
  pub struct FeedPage(ObjectSubclass<imp::FeedPage>)
      @extends gtk::Widget, gtk::Box,
      @implements gtk::Accessible, gtk::Buildable, gtk::Orientable;
}

impl FeedPage {
  // ----------------------------------------------------------------- constructor methods

  pub fn new() -> Self {
    glib::Object::builder().build()
  }

  // ---------------------------------------------------------------------- public methods

  pub fn set_feed(&self, feed: &Feed) {
    feed
      .bind_property("title", &self.imp().feed_name.get(), "text")
      .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
      .build();
    feed
      .bind_property("url", &self.imp().feed_url.get(), "text")
      .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
      .build();
    feed
      .bind_property("filter", &self.imp().feed_filter.get(), "text")
      .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
      .build();

    feed.connect_notify_local(
      Some("url"),
      glib::clone!(@weak self as this => move |feed, _| {
        this.imp().no_url_message.set_visible(feed.get_url().is_empty());
      }),
    );

    self
      .imp()
      .no_url_message
      .set_visible(feed.get_url().is_empty());
  }

  pub fn set_connection_failed(&self) {
    self.imp().connection_error_message.set_visible(true);
    self.imp().view.set_visible(false);
  }

  pub fn set_items(&self, items: &Vec<FeedItem>) {
    self.imp().connection_error_message.set_visible(false);
    self.imp().view.set_visible(true);

    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
      const PADDING: i32 = 12;

      let label = gtk::Label::builder()
        .halign(gtk::Align::Start)
        .hexpand(true)
        .ellipsize(pango::EllipsizeMode::End)
        .margin_top(PADDING)
        .margin_bottom(PADDING)
        .margin_start(PADDING)
        .margin_end(PADDING)
        .build();

      let icon = gtk::Image::builder()
        .margin_top(PADDING)
        .margin_bottom(PADDING)
        .margin_start(PADDING)
        .margin_end(PADDING)
        .icon_name("adw-external-link-symbolic")
        .build();

      let hbox = gtk::Box::builder().build();
      hbox.append(&label);
      hbox.append(&icon);

      list_item.set_activatable(true);
      list_item.set_child(Some(&hbox));
    });

    factory.connect_bind(move |_, list_item| {
      // Get `FeedItem` from `ListItem`
      let feed_item = list_item
        .item()
        .expect("The item has to exist.")
        .downcast::<FeedItem>()
        .expect("The item has to be a `FeedItem`.");

      let title = feed_item.get_title();

      // Get `Label` from `ListItem`
      let label = list_item
        .child()
        .unwrap()
        .downcast::<gtk::Box>()
        .unwrap()
        .first_child()
        .unwrap()
        .downcast::<gtk::Label>()
        .unwrap();

      label.set_label(&title.to_string());
    });

    // Add the vector to the model
    self.imp().model.remove_all();
    self.imp().model.extend_from_slice(&items);

    let filter_model =
      gtk::FilterListModel::new(Some(&self.imp().model), Some(&self.imp().filter));
    let selection_model = gtk::NoSelection::new(Some(&filter_model));
    self.imp().view.set_model(Some(&selection_model));
    self.imp().view.set_factory(Some(&factory));
    self.imp().view.set_css_classes(&["card"]);
  }
}

mod imp {
  use super::*;

  #[derive(Debug, CompositeTemplate)]
  #[template(resource = "/io/github/schneegans/BingeRSS/ui/FeedPage.ui")]
  pub struct FeedPage {
    #[template_child]
    pub feed_name: TemplateChild<adw::EntryRow>,
    #[template_child]
    pub feed_url: TemplateChild<adw::EntryRow>,
    #[template_child]
    pub feed_filter: TemplateChild<adw::EntryRow>,
    #[template_child]
    pub view: TemplateChild<gtk::ListView>,
    #[template_child]
    pub connection_error_message: TemplateChild<adw::StatusPage>,
    #[template_child]
    pub no_url_message: TemplateChild<adw::StatusPage>,

    pub model: gio::ListStore,
    pub filter: gtk::StringFilter,
  }

  impl Default for FeedPage {
    fn default() -> Self {
      Self {
        feed_name: TemplateChild::default(),
        feed_url: TemplateChild::default(),
        feed_filter: TemplateChild::default(),
        view: TemplateChild::default(),
        connection_error_message: TemplateChild::default(),
        no_url_message: TemplateChild::default(),
        model: gio::ListStore::new(FeedItem::static_type()),
        filter: gtk::StringFilter::builder()
          .ignore_case(true)
          .match_mode(gtk::StringFilterMatchMode::Substring)
          .expression(gtk::PropertyExpression::new(
            FeedItem::static_type(),
            gtk::Expression::NONE,
            "title",
          ))
          .build(),
      }
    }
  }

  #[glib::object_subclass]
  impl ObjectSubclass for FeedPage {
    const NAME: &'static str = "FeedPage";
    type Type = super::FeedPage;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
      klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
      obj.init_template();
    }
  }

  impl ObjectImpl for FeedPage {
    fn constructed(&self) {
      self.parent_constructed();

      self
        .view
        .set_cursor(Some(&gdk::Cursor::from_name("pointer", None).unwrap()));

      self.view.connect_activate(|view, pos| {
        let item = view
          .model()
          .unwrap()
          .item(pos)
          .unwrap()
          .downcast::<FeedItem>()
          .unwrap();
        let url = item.get_url();
        let result =
          gio::AppInfo::launch_default_for_uri(&url, gio::AppLaunchContext::NONE);
        if result.is_err() {
          println!("Failed to open URL {}", url);
        }
      });

      self
        .feed_filter
        .connect_changed(glib::clone!(@weak self as this => move |entry| {
          this.obj().imp().filter.set_search(Some(&entry.text()));
        }));
    }
  }

  impl WidgetImpl for FeedPage {}

  impl BoxImpl for FeedPage {}
}
