mod components;
mod sources;

use adw::prelude::*;
use components::FeedItemList;
use gtk::{gdk, gio};

fn main() {
  // Register and include resources
  gio::resources_register_include!("BingeRSS.gresource").expect("register resources");

  let application = adw::Application::builder()
    .application_id("apps.BingeRSS")
    .build();

  //
  application.connect_startup(|_| {
    adw::init();
    let display = gdk::Display::default().expect("get default gdk::Display");
    gtk::IconTheme::for_display(&display).add_resource_path("/apps/BingeRSS");
  });

  let urls = vec![
    "https://www.spiegel.de/schlagzeilen/tops/index.rss",
    "http://reddit.com/r/unixporn/new/.rss?sort=new",
    "https://omgubuntu.co.uk/feed",
    "https://www.blendernation.com/feed/",
    "https://nvd.nist.gov/feeds/xml/cve/misc/nvd-rss-analyzed.xml",
  ];

  application.connect_activate(move |app| {
    let sources_list = gtk::ListBox::builder()
      .css_classes(vec![String::from("content")])
      .build();

    for url in &urls {
      let content = sources::get_feed(url);

      if content.is_err() {
        continue;
      }

      let content = content.unwrap();
      let title = &content.title.as_ref().unwrap().content;
      let subtitle = &content.description.as_ref().unwrap().content;

      let row = adw::ActionRow::builder()
        .activatable(true)
        .selectable(false)
        .title(title)
        .subtitle(subtitle)
        .build();
      sources_list.append(&row);

      // let icon_url = {
      //   if content.logo.is_some() {
      //     &content.logo.as_ref().unwrap().uri
      //   } else if content.icon.is_some() {
      //     &content.icon.as_ref().unwrap().uri
      //   } else {
      //     ""
      //   }
      // };

      let url = url::Url::parse(&content.links[0].href);
      let icon_url = url.as_ref().unwrap().scheme().to_string()
        + &String::from("://")
        + &url.as_ref().unwrap().host().unwrap().to_string()
        + &String::from("/favicon.ico");

      if icon_url != "" {
        let image = sources::get_image(&icon_url);

        if image.is_ok() {
          row.add_prefix(&image.unwrap());
        } else {
          println!(
            "Failed to download image {:?} (reason: {:?})",
            &icon_url, &image
          );
        }
      }

      let subpage = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

      let title = gtk::Label::builder()
        .label(title)
        .css_classes(vec![String::from("title")])
        .build();

      let header = adw::HeaderBar::builder().title_widget(&title).build();
      subpage.append(&header);

      let item_list = FeedItemList::new();

      for item in content.entries {
        let title = if item.title.is_some() {
          item.title.as_ref().unwrap().content.clone()
        } else {
          String::from("Foo")
        };

        let url = item.links[0].href.clone();
        item_list.add_row(title, url);
      }

      subpage.append(&item_list);

      row.connect_activated(move |row: &adw::ActionRow| {
        row
          .root()
          .unwrap()
          .downcast_ref::<adw::PreferencesWindow>()
          .unwrap()
          .present_subpage(&subpage);
      });
    }

    let group = adw::PreferencesGroup::builder()
      .margin_top(32)
      .margin_end(32)
      .margin_bottom(32)
      .margin_start(32)
      .build();
    group.add(&sources_list);

    let sources_page = adw::PreferencesPage::builder()
      .title("Sources")
      .icon_name("apps.BingeRSS.sources-symbolic")
      .build();
    sources_page.add(&group);

    let preferences_page = adw::PreferencesPage::builder()
      .title("Preferences")
      .icon_name("apps.BingeRSS.settings-symbolic")
      .build();

    let window = adw::PreferencesWindow::builder()
      .application(app)
      .title("BingeRSS")
      .default_width(350)
      .can_navigate_back(true)
      .build();

    window.add(&sources_page);
    window.add(&preferences_page);
    window.add_css_class("devel");
    window.show();
  });

  application.run();
}
