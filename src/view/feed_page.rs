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
use gtk::{gdk, gio, glib, CompositeTemplate};

use crate::model::{Feed, FeedItem, FeedState};

// ---------------------------------------------------------------------------------------
// The FeedPage is derived from gtk::Box. There is one FeedPage shown on the right for
// each feed. It shows text entries for the feed's title, URL, and filter as well as the
// actual feed items once downloaded. Depending on the Feed's state, it can also display
// several info messages.
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

  // This assigns a Feed to the FeedPage. The method will bind some properties of the
  // FeedPage to the properties of the Feed.
  pub fn set_feed(&self, feed: &Feed) {
    // Sync the Feed's title to the current value of the title entry field.
    feed
      .bind_property("title", &self.imp().title_entry.get(), "text")
      .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
      .build();

    // Sync the Feed's URL to the current value of the URL entry field.
    feed
      .bind_property("url", &self.imp().url_entry.get(), "text")
      .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
      .build();

    // Sync the Feed's filter to the current value of the filter entry field.
    feed
      .bind_property("filter", &self.imp().filter_entry.get(), "text")
      .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
      .build();

    // Make sure that the actual feed list is filtered whenever the filter value changes.
    feed
      .bind_property("filter", &self.imp().filter, "search")
      .flags(glib::BindingFlags::SYNC_CREATE)
      .build();

    // Depending on the Feed's state, we show and hide several components of the FeedPage.
    feed.connect_notify_local(
      Some("state"),
      glib::clone!(@weak self as this => move |feed, _| {

        this.imp().no_url_message.set_visible(false);
        this.imp().connection_error_message.set_visible(false);
        this.imp().feed_items_group.set_visible(true);

        let state = feed.get_state().clone();

        if state == FeedState::EmptyURL {
          this.imp().no_url_message.set_visible(true);
          this.imp().feed_items_group.set_visible(false);
        } else if state == FeedState::DownloadFailed {
          this.imp().connection_error_message.set_visible(true);
          this.imp().feed_items_group.set_visible(false);
        } else if state == FeedState::DownloadSucceeded {
          this.imp().feed_items_group.set_visible(true);
        }

        // Update the items. We do not want to clear the item list if the download started
        // in order to reduce visual noise.
        if state != FeedState::DownloadStarted {
          this.imp().model.remove_all();
          this.imp().model.extend_from_slice(&feed.get_items().as_ref());
        }
      }),
    );
  }
}

mod imp {
  use super::*;

  // -------------------------------------------------------------------------------------
  // The structure of this custom widget is defined in the FeedPage.ui file.
  #[derive(Debug, CompositeTemplate)]
  #[template(resource = "/io/github/schneegans/BingeRSS/ui/FeedPage.ui")]
  pub struct FeedPage {
    #[template_child]
    pub title_entry: TemplateChild<adw::EntryRow>,
    #[template_child]
    pub url_entry: TemplateChild<adw::EntryRow>,
    #[template_child]
    pub filter_entry: TemplateChild<adw::EntryRow>,
    #[template_child]
    pub feed_items_group: TemplateChild<adw::PreferencesGroup>,
    #[template_child]
    pub feed_item_list_box: TemplateChild<gtk::ListBox>,
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
        title_entry: TemplateChild::default(),
        url_entry: TemplateChild::default(),
        filter_entry: TemplateChild::default(),
        feed_items_group: TemplateChild::default(),
        feed_item_list_box: TemplateChild::default(),
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
    // Most components of this custom widget are defined in the UI file. However, some
    // things have to be set up in code. This is done here, whenever a new FeedPage is
    // constructed.
    fn constructed(&self) {
      self.parent_constructed();

      // Wire up everything.
      let filter_model = gtk::FilterListModel::new(Some(&self.model), Some(&self.filter));
      let slice_model = gtk::SliceListModel::new(Some(&filter_model), 0, 50);
      let selection_model = gtk::NoSelection::new(Some(&slice_model));
      self
        .feed_item_list_box
        .bind_model(Some(&selection_model), move |item| {
          let title: String = item.property("title");

          let row = adw::ActionRow::builder()
            .title(&title)
            .title_lines(1)
            .selectable(false)
            .activatable(true)
            .use_markup(false)
            .build();

          let icon = gtk::Image::builder()
            .icon_name("adw-external-link-symbolic")
            .build();

          row.add_suffix(&icon);

          let url: String = item.property("url");

          row.connect_activated(move |_| {
            let result =
              gio::AppInfo::launch_default_for_uri(&url, gio::AppLaunchContext::NONE);
            if result.is_err() {
              println!("Failed to open URL {}", url);
            }
          });

          // Make the cursor change to a pointer if hovering over the item list. This
          // increases the affordance of clickable links.
          row.set_cursor(Some(&gdk::Cursor::from_name("pointer", None).unwrap()));

          let result = row.ancestor(gtk::Widget::static_type());
          result.unwrap()
        });
    }
  }

  impl WidgetImpl for FeedPage {}

  impl BoxImpl for FeedPage {}
}
