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
use gtk::gio;
use gtk::subclass::prelude::*;
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
    #[template_child]
    pub feed_item_group: TemplateChild<adw::PreferencesGroup>,
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
    self.imp().feed_item_group.set_visible(false);
  }

  pub fn set_items(&self, items: Vec<FeedItem>) {
    self.imp().connection_error.set_visible(false);
    self.imp().feed_item_group.set_visible(true);

    // for item in items {
    //   let row = adw::ActionRow::builder()
    //     .activatable(true)
    //     .title(&glib::markup_escape_text(&item.get_title()))
    //     .title_lines(1)
    //     .selectable(false)
    //     .build();

    //   row.connect_activated(move |_| {
    //     let result =
    //       gio::AppInfo::launch_default_for_uri(&item.get_url(), gio::AppLaunchContext::NONE);
    //     if result.is_err() {
    //       println!("Failed to open URL {}", item.get_url());
    //     }
    //   });

    //   self.imp().feed_items.append(&row);
    // }

    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
      let label = adw::ActionRow::new();
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

      // Get `Label` from `ListItem`
      let label = list_item
        .child()
        .expect("The child has to exist.")
        .downcast::<adw::ActionRow>()
        .expect("The child has to be a `Label`.");

      // Set "label" to "number"
      label.set_title(&title.to_string());
    });

    // Create new model
    let model = gio::ListStore::new(FeedItem::static_type());

    // Add the vector to the model
    model.extend_from_slice(&items);

    let selection_model = gtk::SingleSelection::new(Some(&model));
    self.imp().feed_items.set_model(Some(&selection_model));
    self.imp().feed_items.set_factory(Some(&factory));
  }
}
