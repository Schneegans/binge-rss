use adw::prelude::*;
use glib::subclass::InitializingObject;
use gtk::gio;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use super::FeedItem;

mod imp {
  use super::*;

  #[derive(Debug, CompositeTemplate, Default)]
  #[template(resource = "/io/github/schneegans/BingeRSS/ui/FeedContentPage.ui")]
  pub struct FeedContentPage {
    #[template_child]
    pub feed_items: TemplateChild<gtk::ListBox>,
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

  pub fn set_items(&self, items: Vec<FeedItem>) {
    for item in items {
      let row = adw::ActionRow::builder()
        .activatable(true)
        .title(&item.title)
        .title_lines(1)
        .selectable(false)
        .build();

      row.connect_activated(move |_| {
        let result = gio::AppInfo::launch_default_for_uri(&item.url, gio::AppLaunchContext::NONE);
        if result.is_err() {
          println!("Failed to open URL {}", item.url);
        }
      });

      self.imp().feed_items.append(&row);
    }
  }
}
