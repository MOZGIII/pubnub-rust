//! # Async PubNub Client SDK for Rust
//!
//! - Fully `async`/`await` ready.
//! - Uses Tokio and Hyper to provide an ultra-fast, incredibly reliable message transport over the
//!   PubNub edge network.
//! - Optimizes for minimal network sockets with an infinite number of logical streams.
//!
//! # Example
//!
//! ```
//! use futures_util::stream::StreamExt;
//! use pubnub_hyper::{core::json::object, PubNub};
//!
//! # async {
//! let mut pubnub = PubNub::new("demo", "demo");
//!
//! let message = object!{
//!     "username" => "JoeBob",
//!     "content" => "Hello, world!",
//! };
//!
//! let mut stream = pubnub.subscribe("my-channel").await;
//! let timetoken = pubnub.publish("my-channel", message.clone()).await?;
//!
//! let received = stream.next().await;
//! assert_eq!(received.unwrap().json, message);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # };
//! ```

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![forbid(unsafe_code)]

pub use crate::message::{Message, Timetoken, Type};
pub use crate::pubnub::{PubNub, PubNubBuilder};
pub use crate::runtime::Runtime;
pub use crate::subscription::Subscription;
pub use crate::transport::Transport;

pub use async_trait::async_trait;

mod message;
mod pubnub;
mod runtime;
mod subscription;
mod transport;

#[cfg(test)]
mod tests;
