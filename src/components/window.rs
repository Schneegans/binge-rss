use std::error::Error;

use adw::prelude::*;
use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use super::{FeedItem, FeedItemList};

mod imp {
  use adw::subclass::prelude::AdwApplicationWindowImpl;

  use super::*;

  #[derive(Debug, CompositeTemplate, Default)]
  #[template(resource = "/apps/BingeRSS/ui/Window.ui")]
  pub struct Window {
    #[template_child]
    pub leaflet: TemplateChild<adw::Leaflet>,
    #[template_child]
    pub feed_list: TemplateChild<gtk::ListBox>,
    #[template_child]
    pub add_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub feed_details: TemplateChild<gtk::Stack>,
  }

  #[glib::object_subclass]
  impl ObjectSubclass for Window {
    const NAME: &'static str = "Window";
    type Type = super::Window;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
      klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
      obj.init_template();
    }
  }

  impl ObjectImpl for Window {}

  impl WidgetImpl for Window {}

  impl WindowImpl for Window {}

  impl ApplicationWindowImpl for Window {}

  impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, adw::Window, @implements gtk::Accessible, gtk::Buildable;
}

impl Window {
  pub fn new() -> Self {
    glib::Object::new(&[]).expect("Failed to create Window")
  }

  pub fn add_feed(&self, title: &str, url: &str) {
    let row = adw::ActionRow::builder()
      .activatable(true)
      .selectable(true)
      .title(title)
      .build();
    self.imp().feed_list.append(&row);

    let spinner = gtk::Spinner::new();
    spinner.start();
    row.add_prefix(&spinner);

    let url = url.to_owned();
    let handle = crate::RUNTIME.spawn(async move {
      let content = reqwest::get(url).await?.bytes().await?;
      let feed = feed_rs::parser::parse(&content[..])?;
      Ok::<feed_rs::model::Feed, Box<dyn Error + Send + Sync>>(feed)
    });

    let ctx = glib::MainContext::default();
    ctx.spawn_local(glib::clone!(@weak self as this => async move {
        let content = handle.await.unwrap().unwrap();

        // if false {
        //   let url = url::Url::parse(&content.links[0].href);
        //   let icon_url = url.as_ref().unwrap().scheme().to_string()
        //     + &String::from("://")
        //     + &url.as_ref().unwrap().host().unwrap().to_string()
        //     + &String::from("/favicon.ico");

        //   if icon_url != "" {
        //     let image = sources::get_image(&icon_url);

        //     if image.is_ok() {
        //       row.add_prefix(&image.unwrap());
        //     } else {
        //       println!(
        //         "Failed to download image {:?} (reason: {:?})",
        //         &icon_url, &image
        //       );
        //     }
        //   }
        // }

        {
          let unread_count = 42;
          let label = gtk::Label::builder()
            .label(&unread_count.to_string())
            .valign(gtk::Align::Center)
            .css_classes(vec!["item-count-badge".to_string()])
            .build();

          row.add_suffix(&label);
        }

        let subpage = gtk::ScrolledWindow::builder().build();

        let feed_items: Vec<FeedItem> = content
          .entries
          .iter()
          .map(|item| {
            let title = if item.title.is_some() {
              item.title.as_ref().unwrap().content.clone()
            } else {
              String::from("Foo")
            };

            let url = item.links[0].href.clone();

            FeedItem::new(title, url)
          })
          .collect();

        let item_list = FeedItemList::new();
        item_list.set_items(feed_items);

        subpage.set_child(Some(&item_list));

        this.imp().feed_details.add_child(&subpage);

        row.connect_activated( move |_| {
          this.show_details_page();
          this.imp().feed_details.set_visible_child(&subpage);
        });
    }));
  }

  pub fn show_feed_page(&self) {
    self.imp().leaflet.set_visible_child_name("feed_list_page");
  }

  pub fn show_details_page(&self) {
    self
      .imp()
      .leaflet
      .set_visible_child_name("feed_details_page");
  }
}
