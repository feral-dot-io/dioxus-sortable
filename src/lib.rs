#![warn(missing_docs)]
//!
//! # Sortable components for Dioxus
//!
//! Create sortable tables (and other components) of any type for [Dioxus](https://dioxuslabs.com/).
//!
//! The focus is on tables but can be fully customised to create any type of sortable component. Your tables can be customised however you wish. Sorting state is kept separately from the data.
//!
//! Throughout this documentation, you'll see the type `T` used to refer to the data type that you wish to sort. You'll also see `F` which is expected to be an enum referring to each sortable field of `T`. Your `F` enum should implement [`PartialOrdBy`] to sort and [`Sortable`] to describe how it may be sorted.
//!
//! We use [`PartialOrd`] to allow sorting of types with NULL semantics. This is useful where we have `f64::NAN` or an "unknown" field. It allows us to handle more general cases. We try to keep ordering semantics the same as SQL's ORDER BY clause.
//!
//! ## Usage
//!
//! 1. Create a `struct T` that you wish to sort. The table row.
//! 2. Create an `enum F` that describes each sortable field in `T`.
//! 3. Implement [`PartialOrdBy`] for `F`. This is used to sort `T` by `F`.
//! 4. Implement [`Sortable`] for `F`. This is used to describe how `F` may be sorted.
//! 5. Call [`use_sorter()`] in your component and get a [`UseSorter`].
//! 6. Call [`UseSorter::sort`] to sort data. This may be called conditionally e.g., when waiting for data to arrive.
//! 7. Create a table using [`Th`] or write your own with [`ThStatus`] and [`UseSorter::toggle_field`].
//!
//! ## Example
//!
//! See `examples/prime_ministers.rs` for a complete example. You can modify and run it locally with `dioxus serve --example prime_ministers`
//!
//! ### Minimal example
//!
//! ```rust
//! use dioxus::prelude::*;
//! use dioxus_sortable::{use_sorter, PartialOrdBy, SortBy, Sortable, Th};
//!
//! /// Our table row. Type `T`.
//! #[derive(Clone, Debug, PartialEq)]
//! struct Person {
//!     name: String,
//!     age: u8,
//! }
//!
//! /// Our table columns. Type `F`. One for each field in Person.
//! #[derive(Copy, Clone, Debug, Default, PartialEq)]
//! enum PersonField {
//!     Name,
//!     /// Use default for the initial sort.
//!     #[default]
//!     Age,
//! }
//!
//! /// Specify how we sort our `Person` using `PersonField`.
//! impl PartialOrdBy<Person> for PersonField {
//!     fn partial_cmp_by(&self, a: &Person, b: &Person) -> Option<std::cmp::Ordering> {
//!         // Note how it's just a passthru to `PartialOrd` for each field.
//!         match self {
//!             PersonField::Name => a.name.partial_cmp(&b.name),
//!             PersonField::Age => a.age.partial_cmp(&b.age),
//!         }
//!     }
//! }
//!
//! /// Specify sorting options available on a column.
//! impl Sortable for PersonField {
//!     fn sort_by(&self) -> Option<SortBy> {
//!         // We can choose column specifics but this is good for the minimum.
//!         SortBy::increasing_or_decreasing()
//!     }
//! }
//!
//! #[inline_props]
//! fn OurMinimalExampleTable(cx: Scope) -> Element {
//!     // Set up Dioxus state hooks. *Must* be called every time in the same order
//!     let sorter = use_sorter::<PersonField>(cx);
//!     // Obtain our data. Either passed via props or pulled from a server
//!     let mut data = load_data();
//!     // Sort our data. This is optional but needed to apply the sort
//!     sorter.sort(data.as_mut_slice());
//!
//!     // Render our table like normal.
//!     cx.render(rsx! {
//!         table {
//!             thead {
//!                 tr {
//!                     // Note that we use `Th` instead of `th`.
//!                     // We could customise `th` by using `ThStatus` instead.
//!                     // Or use `UseSorter::toggle_field` elsewhere to lift
//!                     // the sorter out of the table entirely.
//!                     Th { sorter: sorter, field: PersonField::Name, "Name" }
//!                     Th { sorter: sorter, field: PersonField::Age, "Age" }
//!                 }
//!             }
//!             tbody {
//!                 // Iterate on our sorted data.
//!                 // If we didn't want sortable tables, this could easily be a
//!                 // `ul { li { ... } }` instead.
//!                 for person in data.iter() {
//!                     tr {
//!                         td { "{person.name}" }
//!                         td { "{person.age}" }
//!                     }
//!                 }
//!             }
//!         }
//!    })
//! }
//!
//! # fn load_data() -> Vec<Person> {
//! #     vec![
//! #         Person { name: "John".to_string(), age: 32 },
//! #         Person { name: "Jane".to_string(), age: 28 },
//! #         Person { name: "Bob".to_string(), age: 42 },
//! #     ]
//! # }
//! ```
//!

mod rsx;
pub use rsx::*;
mod use_sorter;
pub use use_sorter::*;
