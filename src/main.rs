mod components;

use adw::prelude::*;
use components::Feed;
use components::Window;
use gtk::glib;
use gtk::{gdk, gio};
use once_cell::sync::Lazy;

pub static RUNTIME: Lazy<tokio::runtime::Runtime> =
  Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

fn main() {
  // Register and include resources
  gio::resources_register_include!("compiled.gresource").expect("register resources");

  let application = adw::Application::builder()
    .application_id("io.github.schneegans.BingeRSS")
    .build();

  // asdsas
  application.connect_startup(|_| {
    adw::init();
    let display = gdk::Display::default().expect("get default gdk::Display");
    gtk::IconTheme::for_display(&display).add_resource_path("/io/github/schneegans/BingeRSS");
  });

  application.connect_activate(move |app| {
    let window = Window::new();
    window.set_application(Some(app));
    window.set_title(Some(&"BingeRSS".to_string()));

    window.show();

    setup_actions(&window);

    let data = r#"
      [
        {
          "title": "Der SPIEGEL",
          "url": "https://www.spiegel.de/schlagzeilen/tops/index.rss",
          "viewed": "2022-10-09 16:06:14 UTC"
        },
        {
          "title": "Unixporn",
          "url": "http://reddit.com/r/unixporn/new/.rss?sort=new",
          "viewed": "2022-10-09 16:06:14 UTC"
        },
        {
          "title": "Forschung Aktuell",
          "url": "https://www.deutschlandfunk.de/forschung-aktuell-104.xml",
          "viewed": "2022-10-09 16:06:14 UTC"
        },
        {
          "title": "Linux",
          "url": "http://reddit.com/r/linux/new/.rss?sort=new",
          "viewed": "2022-10-09 16:06:14 UTC"
        },
        {
          "title": "GNOME",
          "url": "http://reddit.com/r/gnome/new/.rss?sort=new",
          "viewed": "2022-10-09 16:06:14 UTC"
        },
        {
          "title": "OMG Ubuntu",
          "url": "https://omgubuntu.co.uk/feed",
          "viewed": "2022-10-09 16:06:14 UTC"
        },
        {
          "title": "Blendernation",
          "url": "https://www.blendernation.com/feed/",
          "viewed": "2022-10-09 16:06:14 UTC"
        },
        {
          "title": "The Verge",
          "url": "https://www.theverge.com/rss/index.xml",
          "viewed": "2022-10-09 16:06:14 UTC"
        },
        {
          "title": "Ars Technica",
          "url": "https://feeds.arstechnica.com/arstechnica/features",
          "viewed": "2022-10-09 16:06:14 UTC"
        },
        {
          "title": "Hacker News",
          "url": "https://news.ycombinator.com/rss",
          "viewed": "2022-10-09 16:06:14 UTC"
        },
        {
          "title": "Vulnerabilities",
          "url": "https://nvd.nist.gov/feeds/xml/cve/misc/nvd-rss-analyzed.xml",
          "viewed": "2022-10-09 16:06:14 UTC"
        }
      ]"#;

    let feeds: Vec<Feed> = serde_json::from_str(data).expect("valid json");

    for feed in feeds {
      window.add_feed(feed);
    }
  });

  application.run();
}

fn setup_actions(window: &Window) {
  {
    let actions = gio::SimpleActionGroup::new();
    window.insert_action_group("app", Some(&actions));

    {
      let action = gio::SimpleAction::new("about", None);
      action.connect_activate(glib::clone!(@weak window => move |_, _| {
        let dialog = gtk::AboutDialog::builder()
          .program_name("BingeRSS")
          .license_type(gtk::License::MitX11)
          .logo_icon_name("io.github.schneegans.BingeRSS")
          .authors(vec!["Simon Schneegans".into()])
          .artists(vec!["Simon Schneegans".into()])
          .website("https://github.com/schneegans/binge-rss")
          .version("0.1.0")
          .transient_for(&window)
          .modal(true)
          .build();

        dialog.present();
      }));

      actions.add_action(&action);
    }

    {
      let action = gio::SimpleAction::new("quit", None);
      action.connect_activate(glib::clone!(@weak window => move |_, _| {
        window.close();
      }));
      actions.add_action(&action);
    }
  }

  {
    let actions = gio::SimpleActionGroup::new();
    window.insert_action_group("feeds", Some(&actions));

    {
      let action = gio::SimpleAction::new("add", None);
      action.connect_activate(move |_, _| {
        println!("add");
      });
      actions.add_action(&action);
    }

    {
      let action = gio::SimpleAction::new("remove", None);
      action.connect_activate(move |_, _| {
        println!("remove");
      });
      actions.add_action(&action);
    }

    {
      let action = gio::SimpleAction::new("show", None);
      action.connect_activate(glib::clone!(@weak window => move |_, _| {
        window.show_feed_page();
      }));
      actions.add_action(&action);
    }
  }
}
