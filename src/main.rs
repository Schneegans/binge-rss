mod components;

use adw::prelude::*;
use components::Window;
use gtk::glib;
use gtk::{gdk, gio};
use once_cell::sync::Lazy;

pub static RUNTIME: Lazy<tokio::runtime::Runtime> =
  Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

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
    gtk::IconTheme::for_display(&display).add_resource_path("/io/github/schneegans/BingeRSS");
  });

  application.connect_activate(move |app| {
    let window = Window::new();
    window.set_application(Some(app));
    window.set_title(Some(&"BingeRSS".to_string()));

    window.add_css_class("devel");
    window.show();

    setup_actions(&window);

    window.add_feed(
      "Der SPIEGEL",
      "https://www.spiegel.de/schlagzeilen/tops/index.rss",
    );
    window.add_feed("Unixporn", "http://reddit.com/r/unixporn/new/.rss?sort=new");
    window.add_feed(
      "Forschung Aktuell",
      "https://www.deutschlandfunk.de/forschung-aktuell-104.xml",
    );
    window.add_feed("Linux", "http://reddit.com/r/linux/new/.rss?sort=new");
    window.add_feed("GNOME", "http://reddit.com/r/gnome/new/.rss?sort=new");
    window.add_feed("OMG Ubuntu", "https://omgubuntu.co.uk/feed");
    window.add_feed("Blendernation", "https://www.blendernation.com/feed/");
    window.add_feed("The Verge", "https://www.theverge.com/rss/index.xml");
    window.add_feed(
      "Ars Technica",
      "https://feeds.arstechnica.com/arstechnica/features",
    );
    window.add_feed("Hacker News", "https://news.ycombinator.com/rss");
    //   window.add_feed(
    //     "Vulnerabilities",
    //     "https://nvd.nist.gov/feeds/xml/cve/misc/nvd-rss-analyzed.xml",
    //   );
  });

  application.run();
}

fn setup_actions(window: &Window) {
  {
    let actions = gio::SimpleActionGroup::new();
    window.insert_action_group("app", Some(&actions));

    {
      let action = gio::SimpleAction::new("about", None);
      action.connect_activate(move |_, _| {
        println!("about");
      });
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
