use std::error::Error;

pub fn get_feed(url: &str) -> Result<rss::Channel, Box<dyn Error>> {
    let content = reqwest::blocking::get(url)?.bytes()?;
    let channel = rss::Channel::read_from(&content[..])?;
    Ok(channel)
}
