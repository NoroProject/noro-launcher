//! Sign-in through external platforms: Discord, Twitch, Google.
//!
//! A platform link is a row in `user_identities`. An account can hold several
//! of them and sign in with any one.

pub mod config;
pub mod flow;
pub mod link;
pub mod methods;
pub mod provider;
pub mod remote;
pub mod states;

pub use provider::Provider;
