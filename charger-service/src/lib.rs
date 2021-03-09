#![cfg_attr(not(feature = "std"), no_std)]

pub mod runtime;

#[cfg(feature = "std")]
mod api;

#[cfg(feature = "std")]
mod mock;
