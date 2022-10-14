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
use gtk::{gio, pango};
use gtk::{glib, CompositeTemplate};

use crate::model::FeedItem;

mod imp {
  use super::*;

  #[derive(Debug, CompositeTemplate, Default)]
  #[template(resource = "/io/github/schneegans/BingeRSS/ui/FeedContentPage.ui")]
  pub struct FeedContentPage {
    #[template_child]
    pub feed_items: TemplateChild<gtk::ListView>,
    #[template_child]
    pub connection_error: TemplateChild<adw::StatusPage>,
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

  impl ObjectImpl for FeedContentPage {}

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
    self.imp().feed_items.set_visible(false);
  }

  pub fn set_items(&self, items: Vec<FeedItem>) {
    self.imp().connection_error.set_visible(false);
    self.imp().feed_items.set_visible(true);

    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
      let label = gtk::Label::builder()
        .halign(gtk::Align::Start)
        .ellipsize(pango::EllipsizeMode::End)
        .margin_bottom(8)
        .build();
      list_item.set_activatable(false);
      list_item.set_child(Some(&label));
    });

    factory.connect_bind(move |_, list_item| {
      // Get `FeedItem` from `ListItem`
      let feed_item = list_item
        .item()
        .expect("The item has to exist.")
        .downcast::<FeedItem>()
        .expect("The item has to be a `FeedItem`.");

      let title = feed_item.property::<String>("title");
      let url = feed_item.property::<String>("url");

      // Get `Label` from `ListItem`
      let label = list_item
        .child()
        .expect("The child has to exist.")
        .downcast::<gtk::Label>()
        .expect("The child has to be a `Label`.");

      label.set_markup(&format!(
        "<a href='{}'>{}</a>",
        glib::markup_escape_text(&url.to_string()),
        glib::markup_escape_text(&title.to_string())
      ));
    });

    // Create new model
    let model = gio::ListStore::new(FeedItem::static_type());

    // Add the vector to the model
    model.extend_from_slice(&items);

    let selection_model = gtk::NoSelection::new(Some(&model));
    self.imp().feed_items.set_model(Some(&selection_model));
    self.imp().feed_items.set_factory(Some(&factory));
    self.imp().feed_items.set_css_classes(&["background"]);
  }
}
