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
use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, pango};
use gtk::{glib, CompositeTemplate};

use crate::model::FeedItem;

mod imp {
  use super::*;

  #[derive(Debug, CompositeTemplate)]
  #[template(resource = "/io/github/schneegans/BingeRSS/ui/FeedContentPage.ui")]
  pub struct FeedContentPage {
    #[template_child]
    pub view: TemplateChild<gtk::ListView>,
    #[template_child]
    pub connection_error: TemplateChild<adw::StatusPage>,

    pub model: gio::ListStore,
  }

  impl Default for FeedContentPage {
    fn default() -> Self {
      Self {
        view: TemplateChild::default(),
        connection_error: TemplateChild::default(),
        model: gio::ListStore::new(FeedItem::static_type()),
      }
    }
  }

  #[glib::object_subclass]
  impl ObjectSubclass for FeedContentPage {
    const NAME: &'static str = "FeedContentPage";
    type Type = super::FeedContentPage;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
      klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
      obj.init_template();
    }
  }

  impl ObjectImpl for FeedContentPage {
    fn constructed(&self, obj: &Self::Type) {
      self.parent_constructed(obj);

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
        let url = item.property::<String>("url");
        let result = gio::AppInfo::launch_default_for_uri(&url, gio::AppLaunchContext::NONE);
        if result.is_err() {
          println!("Failed to open URL {}", url);
        }
      });
    }
  }

  impl WidgetImpl for FeedContentPage {}

  impl BoxImpl for FeedContentPage {}
}

glib::wrapper! {
    pub struct FeedContentPage(ObjectSubclass<imp::FeedContentPage>)
        @extends gtk::Widget, gtk::Box, @implements gtk::Accessible, gtk::Buildable, gtk::Orientable;
}

impl FeedContentPage {
  pub fn new() -> Self {
    glib::Object::new(&[]).expect("Failed to create FeedContentPage")
  }

  pub fn set_connection_failed(&self) {
    self.imp().connection_error.set_visible(true);
    self.imp().view.set_visible(false);
  }

  pub fn set_items(&self, items: Vec<FeedItem>) {
    self.imp().connection_error.set_visible(false);
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

      let title = feed_item.property::<String>("title");

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

    let selection_model = gtk::NoSelection::new(Some(&self.imp().model));
    self.imp().view.set_model(Some(&selection_model));
    self.imp().view.set_factory(Some(&factory));
    self.imp().view.set_css_classes(&["card"]);
  }
}
