//////////////////////////////////////////////////////////////////////////////////////////
//                           ___ _               ___  ___ ___                           //
//                          | _ |_)_ _  __ _ ___| _ \/ __/ __|                          //
//                          | _ \ | ' \/ _` / -_)   /\__ \__ \                          //
//                          |___/_|_||_\__, \___|_|_\|___/___/                          //
//                                     |___/                                            //
//////////////////////////////////////////////////////////////////////////////////////////

// SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
// SPDX-License-Identifier: MIT

mod app;
mod config;
mod model;
mod view;

use app::Application;

use adw::prelude::*;
use gtk::{gdk, gio};
use once_cell::sync::Lazy;

// This is used for asynchronous code.
pub static RUNTIME: Lazy<tokio::runtime::Runtime> =
  Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

fn main() {
  // Register and include resources
  gio::resources_register_include!("compiled.gresource").expect("register resources");

  // Initialize GTK and libadwaita.
  gtk::init().expect("Failed to initialize GTK");
  adw::init().expect("Failed to initialize ADW");

  // Add our custom icons to the icon theme.
  let display = gdk::Display::default().expect("get default gdk::Display");
  gtk::IconTheme::for_display(&display)
    .add_resource_path("/io/github/schneegans/BingeRSS");

  // Create the app and run it!
  let application = Application::new();
  application.set_resource_base_path(Some("/io/github/schneegans/BingeRSS"));
  application.run();
}
