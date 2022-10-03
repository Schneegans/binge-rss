use std::error::Error;

use adw::prelude::*;
use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate};

use super::{Feed, FeedContentPage, FeedItem};

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

  impl ObjectImpl for Window {
    fn constructed(&self, obj: &Self::Type) {
      self.parent_constructed(obj);

      self.feed_list.set_sort_func(|a, b| -> gtk::Ordering {
        let a = a.downcast_ref::<adw::ActionRow>().unwrap().title();
        let b = b.downcast_ref::<adw::ActionRow>().unwrap().title();

        if a < b {
          gtk::Ordering::Smaller
        } else if a > b {
          gtk::Ordering::Larger
        } else {
          gtk::Ordering::Equal
        }
      });
    }
  }

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
      .sensitive(false)
      .title(title)
      .build();
    self.imp().feed_list.append(&row);

    let spinner = gtk::Spinner::new();
    spinner.start();
    row.add_prefix(&spinner);

    let url = url.to_owned();
    let handle = crate::RUNTIME.spawn(async move {
      let bytes = reqwest::get(url).await?.bytes().await?;
      let content = feed_rs::parser::parse(&bytes[..])?;

      let mut feed = Feed {
        items: vec![],
        image: None,
      };

      feed.items = content
        .entries
        .iter()
        .map(|item| {
          let title = if item.title.is_some() {
            item.title.as_ref().unwrap().content.clone()
          } else {
            String::from("Foo")
          };

          let url = item.links[0].href.clone();

          FeedItem {
            title: title,
            url: url,
            content: String::new(),
          }
        })
        .collect();

      let url = url::Url::parse(&content.links[0].href);
      let icon_url = url.as_ref().unwrap().scheme().to_string()
        + &String::from("://")
        + &url.as_ref().unwrap().host().unwrap().to_string()
        + &String::from("/favicon.ico");

      let bytes = reqwest::get(icon_url).await?.bytes().await?;
      feed.image = Some(glib::Bytes::from(&bytes.to_vec()));

      Ok::<Feed, Box<dyn Error + Send + Sync>>(feed)
    });

    let ctx = glib::MainContext::default();
    ctx.spawn_local(glib::clone!(@weak self as this => async move {
      let feed = handle.await.unwrap().unwrap();

      row.remove(&spinner);
      row.set_sensitive(true);

      if feed.image.is_some() {
        let stream = gio::MemoryInputStream::from_bytes(&feed.image.unwrap());
        let pixbuf = gdk::gdk_pixbuf::Pixbuf::from_stream(&stream, gio::Cancellable::NONE);

        if pixbuf.is_ok() {
          let image = gtk::Image::from_pixbuf(Some(&pixbuf.unwrap()));
          row.add_prefix(&image);
        }
      }

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

      let item_list = FeedContentPage::new();
      item_list.set_items(feed.items);

      subpage.set_child(Some(&item_list));

      this.imp().feed_details.add_child(&subpage);

      if this.imp().feed_details.first_child() == this.imp().feed_details.last_child() {
        this.imp().feed_list.select_row(Some(&row));
      }

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
