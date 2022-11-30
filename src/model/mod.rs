// ------------------------------------------------------------------------------------ //
//                           ___ _               ___  ___ ___                           //
//                          | _ |_)_ _  __ _ ___| _ \/ __/ __|                          //
//                          | _ \ | ' \/ _` / -_)   /\__ \__ \                          //
//                          |___/_|_||_\__, \___|_|_\|___/___/                          //
//                                     |___/                                            //
// ------------------------------------------------------------------------------------ //

// SPDX-FileCopyrightText: Simon Schneegans <code@simonschneegans.de>
// SPDX-License-Identifier: MIT

// This module contains some objects which store the data which is shown in the user
// interface.

mod feed;
mod feed_item;

pub use self::feed::Feed;
pub use self::feed::FeedState;
pub use self::feed::StoredFeed;
pub use self::feed_item::FeedItem;
