use adw::prelude::*;
use glib::subclass::InitializingObject;
use gtk::gio;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use super::FeedItem;

mod imp {
  use super::*;

  #[derive(Debug, CompositeTemplate, Default)]
  #[template(resource = "/apps/BingeRSS/ui/FeedItemList.ui")]
  pub struct FeedItemList {
    #[template_child]
    pub feed_items: TemplateChild<gtk::ListBox>,
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

  pub fn set_items(&self, items: Vec<FeedItem>) {
    // Create new model
    let model = gio::ListStore::new(FeedItem::static_type());

    // Add the vector to the model
    model.extend_from_slice(&items);

    self.imp().feed_items.bind_model(Some(&model), |item| {
      let title = item.property::<String>("title");
      let url = item.property::<String>("url");

      let row = adw::ActionRow::builder()
        .activatable(true)
        .title(&title)
        .selectable(false)
        .build();

      row.connect_activated(move |_| {
        let result = gio::AppInfo::launch_default_for_uri(&url, gio::AppLaunchContext::NONE);
        if result.is_err() {
          println!("Failed to open URL {}", url);
        }
      });

      row.upcast::<gtk::Widget>()
    });

    /*
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

    let selection_model = gtk::SingleSelection::new(Some(&model));
    self.imp().list.set_model(Some(&selection_model));
    self.imp().list.set_factory(Some(&factory));
    */
  }
}
