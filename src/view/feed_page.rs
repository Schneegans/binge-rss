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

        let state = feed.get_state().clone();

        if state == FeedState::EmptyURL {
          this.imp().stack.set_visible_child_name("no_url_message");
        } else if state == FeedState::DownloadStarted {
          this.imp().stack.set_visible_child_name("spinner");
        } else if state == FeedState::DownloadFailed {
          this.imp().stack.set_visible_child_name("connection_error_message");
        } else if state == FeedState::DownloadSucceeded {
          this.imp().stack.set_visible_child_name("feed_items");
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
    pub stack: TemplateChild<gtk::Stack>,
    #[template_child]
    pub feed_item_list_box: TemplateChild<gtk::ListBox>,

    pub model: gio::ListStore,
    pub filter: gtk::StringFilter,
  }

  impl Default for FeedPage {
    fn default() -> Self {
      Self {
        title_entry: TemplateChild::default(),
        url_entry: TemplateChild::default(),
        filter_entry: TemplateChild::default(),
        stack: TemplateChild::default(),
        feed_item_list_box: TemplateChild::default(),
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

      // Wire up everything. We show at most 50 rows, else the performance will degrade
      // too much. We could use a gtk::ListView, however this would require nesting
      // directly inside a gtk::ScrolledWindow which in turn would require a redesign of
      // the user interface.
      let filter_model = gtk::FilterListModel::new(Some(&self.model), Some(&self.filter));
      let slice_model = gtk::SliceListModel::new(Some(&filter_model), 0, 50);
      self
        .feed_item_list_box
        .bind_model(Some(&slice_model), move |item| {
          // The item's title is shown on each row.
          let title: String = item.property("title");
          let row = adw::ActionRow::builder()
            .title(&title)
            .title_lines(2)
            .selectable(false)
            .activatable(true)
            .use_markup(false)
            .build();

          // Add an icon as suffix to each row.
          let icon = gtk::Image::builder()
            .icon_name("adw-external-link-symbolic")
            .build();
          row.add_suffix(&icon);

          // Open the item's URL if the row is activated.
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

          row.ancestor(gtk::Widget::static_type()).unwrap()
        });
    }
  }

  impl WidgetImpl for FeedPage {}

  impl BoxImpl for FeedPage {}
}
