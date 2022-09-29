use adw::prelude::*;
use glib::subclass::InitializingObject;
use gtk::gio;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
  use super::*;

  #[derive(Debug, CompositeTemplate, Default)]
  #[template(resource = "/apps/BingeRSS/ui/FeedItemList.ui")]
  pub struct FeedItemList {
    #[template_child]
    pub list: TemplateChild<gtk::ListBox>,
  }

  #[glib::object_subclass]
  impl ObjectSubclass for FeedItemList {
    const NAME: &'static str = "FeedItemList";
    type Type = super::FeedItemList;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
      klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
      obj.init_template();
    }
  }

  impl ObjectImpl for FeedItemList {}
  // Trait shared by all widgets
  impl WidgetImpl for FeedItemList {}

  impl BoxImpl for FeedItemList {}
}

glib::wrapper! {
    pub struct FeedItemList(ObjectSubclass<imp::FeedItemList>)
        @extends gtk::Widget, gtk::Box, @implements gtk::Accessible, gtk::Buildable, gtk::Orientable;
}

impl FeedItemList {
  pub fn new() -> Self {
    glib::Object::new(&[]).expect("Failed to create FeedItemList")
  }

  pub fn add_row(&self, title: String, url: String) {
    let row = adw::ActionRow::builder()
      .activatable(true)
      .title(&title)
      .selectable(false)
      .build();
    self.imp().list.append(&row);

    row.connect_activated(move |_| {
      let result = gio::AppInfo::launch_default_for_uri(&url, gio::AppLaunchContext::NONE);
      if result.is_err() {
        println!("Failed to open URL {}", url);
      }
    });
  }
}
