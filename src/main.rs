use adw::prelude::*;

use gtk::{gdk, gio};

fn main() {
    // Register and include resources
    gio::resources_register_include!("BingeRSS.gresource").expect("Failed to register resources.");

    let application = adw::Application::builder()
        .application_id("apps.BingeRSS")
        .build();
    application.connect_startup(|_| {
        adw::init();

        gtk::IconTheme::for_display(&gdk::Display::default().unwrap()).add_resource_path("/");
    });

    application.connect_activate(|app| {
        let row = adw::ActionRow::builder()
            .activatable(true)
            .selectable(false)
            .title("Click me")
            .build();
        row.connect_activated(|_| {
            println!("Clicked!");
        });

        let list = gtk::ListBox::builder()
            .margin_top(32)
            .margin_end(32)
            .margin_bottom(32)
            .margin_start(32)
            .css_classes(vec![String::from("content")])
            .build();
        list.append(&row);

        let group = adw::PreferencesGroup::new();
        group.add(&list);

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
            .build();

        window.add(&sources_page);
        window.add(&preferences_page);
        window.show();
    });

    application.run();
}
