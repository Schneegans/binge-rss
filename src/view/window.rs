// ------------------------------------------------------------------------------------ //
//                           ___ _               ___  ___ ___                           //
//                          | _ |_)_ _  __ _ ___| _ \/ __/ __|                          //
//                          | _ \ | ' \/ _` / -_)   /\__ \__ \                          //
//                          |___/_|_||_\__, \___|_|_\|___/___/                          //
//                                     |___/                                            //
// ------------------------------------------------------------------------------------ //

// SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
// SPDX-License-Identifier: MIT

use std::error::Error;

use adw::prelude::*;
use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate};

use crate::model::FeedItem;
use crate::view::FeedContentPage;

glib::wrapper! {
  pub struct Window(ObjectSubclass<imp::Window>)
      @extends gtk::Widget, gtk::Window, adw::Window,
      @implements gtk::Accessible, gtk::Buildable;
}

impl Window {
  // ----------------------------------------------------------------- constructor methods

  pub fn new() -> Self {
    glib::Object::new(&[]).expect("Failed to create Window")
  }

  // ---------------------------------------------------------------------- public methods

  pub fn get_selected_id(&self) -> Option<String> {
    match self.imp().feed_list.selected_row() {
      None => None,
      Some(row) => Some(row.widget_name().as_str().to_string()),
    }
  }

  pub fn get_selected_title(&self) -> Option<String> {
    match self.imp().feed_list.selected_row() {
      None => None,
      Some(row) => Some(
        row
          .downcast::<adw::ActionRow>()
          .unwrap()
          .title()
          .as_str()
          .to_string(),
      ),
    }
  }

  pub fn add_feed(&self, id: String, title: String, url: String) {
    self.imp().no_feeds_message.set_visible(false);
    self.imp().leaflet.set_can_unfold(true);

    let row = adw::ActionRow::builder()
      .activatable(true)
      .selectable(true)
      .name(&id)
      .title(&title)
      .build();
    self.imp().feed_list.append(&row);

    let spinner = gtk::Spinner::new();
    spinner.start();
    row.add_prefix(&spinner);

    let item_list = FeedContentPage::new();
    self.imp().feed_details.add_named(&item_list, Some(&id));

    row.connect_activated(
      glib::clone!(@weak self as this, @weak item_list => move |row| {
        this.show_details_page();
        this.imp().feed_details.set_visible_child(&item_list);
        this.imp().header_label.set_label(&row.title());
      }),
    );

    self.imp().feed_list.select_row(Some(&row));
    self.imp().feed_details.set_visible_child(&item_list);
    self.imp().header_label.set_label(&row.title());

    let handle = crate::RUNTIME.spawn(async move {
      let bytes = reqwest::get(&url).await?.bytes().await?;
      let content = feed_rs::parser::parse(&bytes[..])?;

      let url = url::Url::parse(&content.links[0].href);
      let icon_url = url.as_ref().unwrap().scheme().to_string()
        + &String::from("://")
        + &url.as_ref().unwrap().host().unwrap().to_string()
        + &String::from("/favicon.ico");

      let bytes = reqwest::get(icon_url).await?.bytes().await?;
      let image = Some(glib::Bytes::from(&bytes.to_vec()));

      Ok::<(feed_rs::model::Feed, Option<glib::Bytes>), Box<dyn Error + Send + Sync>>((
        content, image,
      ))
    });

    let ctx = glib::MainContext::default();
    ctx.spawn_local(glib::clone!(@weak self as this => async move {

      let feed = handle.await.unwrap();

      row.remove(&spinner);

      let avatar = adw::Avatar::builder()
        .text(&title)
        .size(24)
        .icon_name("rss-symbolic")
        .build();

      row.add_prefix(&avatar);

      if feed.is_ok() {

        let (content, image) = feed.unwrap();

        let items = content.entries
        .iter()
        .map(|item| {
          let title = if item.title.is_some() {
            item.title.as_ref().unwrap().content.clone()
          } else {
            String::from("Unnamed Item")
          };

          let url = item.links[0].href.clone();

          FeedItem::new(title, url)
        })
        .collect();

        if image.is_some() {
          let stream = gio::MemoryInputStream::from_bytes(&image.unwrap());
          let pixbuf = gdk::gdk_pixbuf::Pixbuf::from_stream(&stream, gio::Cancellable::NONE);

          if pixbuf.is_ok() {
            let image = gtk::Image::from_pixbuf(Some(&pixbuf.unwrap()));
            avatar.set_custom_image(Some(&image.paintable().unwrap()));
          }
        }

        {
          let unread_count = 43;
          let label = gtk::Label::builder()
            .label(&unread_count.to_string())
            .valign(gtk::Align::Center)
            .css_classes(vec!["item-count-badge".to_string()])
            .build();

          row.add_suffix(&label);
        }

        item_list.set_items(items);

      } else {
        avatar.set_icon_name(Some("network-no-route-symbolic"));
        row.set_subtitle("Connection failed.");
        item_list.set_connection_failed();
      }
    }));
  }

  pub fn remove_selected_feed(&self) -> Option<String> {
    let list = &self.imp().feed_list;
    let row = list.selected_row()?;
    let id = row.property::<String>("name");

    let mut next_row: Option<gtk::ListBoxRow> = None;

    if row.next_sibling().is_some() {
      next_row = Some(
        row
          .next_sibling()
          .unwrap()
          .downcast::<gtk::ListBoxRow>()
          .unwrap(),
      );
    } else if row.prev_sibling().is_some() {
      next_row = Some(
        row
          .prev_sibling()
          .unwrap()
          .downcast::<gtk::ListBoxRow>()
          .unwrap(),
      );
    }

    list.remove(&row);

    let page = self.get_feed_content_page(&id)?;
    self.imp().feed_details.remove(&page);

    self.imp().header_label.set_label("");

    if next_row.is_some() {
      next_row.unwrap().activate();
    } else {
      self.imp().no_feeds_message.set_visible(true);
      self.imp().leaflet.set_can_unfold(false);
      self.show_feed_page();
    }

    if self.imp().leaflet.is_folded() {
      self.show_feed_page();
    }

    Some(id)
  }

  pub fn set_filter(&self, id: &String, filter: &String) {
    self.get_feed_content_page(id).unwrap().set_filter(filter);
  }

  pub fn get_filter(&self, id: &String) -> String {
    self.get_feed_content_page(id).unwrap().get_filter()
  }

  pub fn get_new_feed_title(&self) -> String {
    self.imp().new_feed_title.text().to_string()
  }

  pub fn get_new_feed_url(&self) -> String {
    self.imp().new_feed_url.text().to_string()
  }

  pub fn show_feed_page(&self) {
    self
      .imp()
      .leaflet
      .set_visible_child(&self.imp().feed_list_page.get());
  }

  pub fn show_details_page(&self) {
    self
      .imp()
      .leaflet
      .set_visible_child(&self.imp().feed_details_page.get());
  }

  // --------------------------------------------------------------------- private methods

  fn get_feed_content_page(&self, id: &String) -> Option<FeedContentPage> {
    let page = self.imp().feed_details.child_by_name(id.as_str());

    if page.is_some() {
      Some(page?.downcast::<FeedContentPage>().unwrap())
    } else {
      None
    }
  }

  fn save_window_size(&self) -> Result<(), glib::BoolError> {
    let (width, height) = self.default_size();

    self.imp().settings.set_int("window-width", width)?;
    self.imp().settings.set_int("window-height", height)?;

    self
      .imp()
      .settings
      .set_boolean("is-maximized", self.is_maximized())?;

    Ok(())
  }

  fn load_window_size(&self) {
    let width = self.imp().settings.int("window-width");
    let height = self.imp().settings.int("window-height");
    let is_maximized = self.imp().settings.boolean("is-maximized");

    self.set_default_size(width, height);

    if is_maximized {
      self.maximize();
    }
  }
}

mod imp {
  use adw::subclass::prelude::AdwApplicationWindowImpl;

  use crate::config;

  use super::*;

  #[derive(Debug, CompositeTemplate)]
  #[template(resource = "/io/github/schneegans/BingeRSS/ui/Window.ui")]
  pub struct Window {
    #[template_child]
    pub leaflet: TemplateChild<adw::Leaflet>,
    #[template_child]
    pub feed_list_page: TemplateChild<gtk::Box>,
    #[template_child]
    pub feed_details_page: TemplateChild<gtk::Box>,
    #[template_child]
    pub feed_list: TemplateChild<gtk::ListBox>,
    #[template_child]
    pub add_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub header_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub feed_details: TemplateChild<gtk::Stack>,
    #[template_child]
    pub new_feed_title: TemplateChild<gtk::Entry>,
    #[template_child]
    pub new_feed_url: TemplateChild<gtk::Entry>,
    #[template_child]
    pub no_feeds_message: TemplateChild<adw::StatusPage>,
    pub settings: gio::Settings,
  }

  impl Default for Window {
    fn default() -> Self {
      Self {
        leaflet: TemplateChild::default(),
        feed_list_page: TemplateChild::default(),
        feed_details_page: TemplateChild::default(),
        feed_list: TemplateChild::default(),
        add_button: TemplateChild::default(),
        header_label: TemplateChild::default(),
        feed_details: TemplateChild::default(),
        new_feed_title: TemplateChild::default(),
        new_feed_url: TemplateChild::default(),
        no_feeds_message: TemplateChild::default(),
        settings: gio::Settings::new(config::APP_ID),
      }
    }
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

      obj.load_window_size();

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

  impl WindowImpl for Window {
    fn close_request(&self, window: &Self::Type) -> gtk::Inhibit {
      if let Err(err) = window.save_window_size() {
        println!("Failed to save window state, {}", &err);
      }

      // Pass close request on to the parent
      self.parent_close_request(window)
    }
  }

  impl ApplicationWindowImpl for Window {}

  impl AdwApplicationWindowImpl for Window {}
}
