#![warn(missing_docs)]
//! # Sortable tables for Dioxus
//!
//! This crate provides a hooks and components for rendering sortable tables in [Dioxus](https://dioxuslabs.com/). It can be used to sort more than just tables.
//!
//! Given a struct `T` and an enum `F` that describes each sortable field in `T`, this crate provides a `use_sorter` hook that can be used to sort `T` by `F`. It also provides a set of components for rendering sortable tables easily.
//!
//! ## Usage
//!
//! 1. Create a `struct T` that you wish to sort. The table row.
//! 2. Create an `enum F` that describes each sortable field in `T`.
//! 3. Implement [`PartialOrdBy`] for `F`. This is used to sort `T` by `F`.
//! 4. Implement [`Sortable`] for `F`. This is used to describe how `F` should be sorted.
//! 5. Call [`use_sorter()`] in your component and get a [`UseSorter`].
//! 6. Call [`UseSorter::sort`] to sort data. This may be called conditionally.
//! 7. Create a table using [`Th`] or write your own with [`ThStatus`] and [`UseSorter::toggle_field`].
//!
//! ## Example
//!
//! TODO:
//! - make a better `examples/table.rs`
//! - provide a minimal example here
//!
//!

mod rsx;
pub use rsx::*;
mod use_sorter;
pub use use_sorter::*;
