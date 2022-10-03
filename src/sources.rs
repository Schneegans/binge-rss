use gtk::{gdk, gio, glib};

use std::error::Error;

pub fn get_image(url: &str) -> Result<gtk::Image, Box<dyn Error>> {
  let content = reqwest::blocking::get(url)?.bytes()?;
  let bytes = glib::Bytes::from(&content.to_vec());
  let stream = gio::MemoryInputStream::from_bytes(&bytes);
  let pixbuf = gdk::gdk_pixbuf::Pixbuf::from_stream(&stream, gio::Cancellable::NONE)?;
  let image = gtk::Image::from_pixbuf(Some(&pixbuf));

  Ok(image)
}
