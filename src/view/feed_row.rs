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
use gtk::{glib, CompositeTemplate};

use crate::model::{Feed, FeedState};

// ---------------------------------------------------------------------------------------
// The FeedRow is derived from adw::ActionRow. There is one FeedRow shown in the sidebar
// on the left for each feed. It shows the feed's title and icon. Depending on the Feed's
// state, it can show a spinner and several error messages.
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

  // This assigns a Feed to the FeedRow. The method will bind some properties of the
  // FeedRow to the properties of the Feed.
  pub fn set_feed(&self, feed: &Feed) {
    // Show the Feed's title.
    feed
      .bind_property("title", self, "title")
      .flags(glib::BindingFlags::SYNC_CREATE)
      .build();

    // Also use the Feed's title for the avatar icon. This will make sure that each icon
    // which does not show a feed icon gets assigned a unique color.
    feed
      .bind_property("title", &self.imp().avatar.get(), "text")
      .flags(glib::BindingFlags::SYNC_CREATE)
      .build();

    // Change the subtitle and the avatar icon based on the Feed's state. This also shows
    // or hides the spinner depending of the downloading-state.
    feed.connect_notify_local(
      Some("state"),
      glib::clone!(@weak self as this => move |feed, _| {

        let state = feed.get_state().clone();

        this.imp().spinner.set_visible(state == FeedState::DownloadStarted);
        this.imp().avatar.set_visible(state != FeedState::DownloadStarted);
        this.imp().badge.set_visible(state == FeedState::DownloadSucceeded);
        this.imp().avatar.set_custom_image(feed.get_icon().as_ref());
        this.imp().avatar.set_icon_name(Some("network-no-route-symbolic"));
        this.set_subtitle("");

        if state == FeedState::DownloadFailed {
          this.set_subtitle("Connection failed");
        } else if state == FeedState::EmptyURL {
          this.set_subtitle("Empty URL");
        } else if state == FeedState::DownloadSucceeded {
          this.imp().avatar.set_icon_name(Some("rss-symbolic"));
        }
      }),
    );
  }
}

mod imp {
  use super::*;

  // -------------------------------------------------------------------------------------

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

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
      obj.init_template();
    }
  }

  impl ObjectImpl for FeedRow {}
  impl WidgetImpl for FeedRow {}
  impl ListBoxRowImpl for FeedRow {}
  impl PreferencesRowImpl for FeedRow {}
  impl ActionRowImpl for FeedRow {}
}
