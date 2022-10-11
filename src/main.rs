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

pub static RUNTIME: Lazy<tokio::runtime::Runtime> =
  Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

fn main() {
  // Register and include resources
  gio::resources_register_include!("compiled.gresource").expect("register resources");

  gtk::init().expect("Failed to initialize GTK");

  adw::init();

  let display = gdk::Display::default().expect("get default gdk::Display");
  gtk::IconTheme::for_display(&display).add_resource_path("/io/github/schneegans/BingeRSS");

  let application = Application::new();
  application.run();
}
