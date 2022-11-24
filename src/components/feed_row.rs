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
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::{glib, CompositeTemplate};

use crate::components::Feed;

glib::wrapper! {
  pub struct FeedRow(ObjectSubclass<imp::FeedRow>)
      @extends adw::ActionRow, adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
      @implements gtk::Accessible, gtk::Buildable, gtk::Actionable;
}

impl FeedRow {
  // ----------------------------------------------------------------- constructor methods

  pub fn new() -> Self {
    glib::Object::builder().build()
  }

  // ---------------------------------------------------------------------- public methods

  pub fn set_feed(&self, feed: &Feed) {
    feed
      .bind_property("title", self, "title")
      .flags(glib::BindingFlags::SYNC_CREATE)
      .build();
    feed
      .bind_property("title", &self.imp().avatar.get(), "text")
      .flags(glib::BindingFlags::SYNC_CREATE)
      .build();
  }

  pub fn set_connection_failed(&self, failed: bool) {
    self.imp().avatar.set_icon_name(Some(if failed {
      "network-no-route-symbolic"
    } else {
      "rss-symbolic"
    }));
    self.set_subtitle(if failed { "Connection failed." } else { "" });
  }
}

mod imp {
  use super::*;

  #[derive(Debug, CompositeTemplate)]
  #[template(resource = "/io/github/schneegans/BingeRSS/ui/FeedRow.ui")]
  pub struct FeedRow {
    #[template_child]
    pub spinner: TemplateChild<gtk::Spinner>,
    #[template_child]
    pub avatar: TemplateChild<adw::Avatar>,
    #[template_child]
    pub badge: TemplateChild<gtk::Label>,
  }

  impl Default for FeedRow {
    fn default() -> Self {
      Self {
        spinner: TemplateChild::default(),
        avatar: TemplateChild::default(),
        badge: TemplateChild::default(),
      }
    }
  }

  #[glib::object_subclass]
  impl ObjectSubclass for FeedRow {
    const NAME: &'static str = "FeedRow";
    type Type = super::FeedRow;
    type ParentType = adw::ActionRow;

    fn class_init(klass: &mut Self::Class) {
      klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
      obj.init_template();
    }
  }

  impl ObjectImpl for FeedRow {}

  impl WidgetImpl for FeedRow {}

  impl ListBoxRowImpl for FeedRow {}

  impl PreferencesRowImpl for FeedRow {}

  impl ActionRowImpl for FeedRow {}
}
