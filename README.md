# Sortable tables for Dioxus

This small library helps creates sortable components for Dioxus apps. It's focus is on tables but isn't limited to them.

1. Create a `struct T`.
2. Create a `enum F` that describes each sortable field in `T`.
3. Implement `PartialOrdBy<T>` for `F`.
4. Implement `Sortable` for `F`.
5. Call `use_sorter` in your component and get a `UseSorter`.
6. Call `UseSorter::sort<T>` to sort data.
7. Create a table using `Th` or write your own with `ThStatus` and `UseSorter::set_field`.

You're done! Let's describe the traits in more detail with an example.

## Example

The first trait `PartialOrdBy<T>` expects you to call `PartialOrd::partial_cmp` on the field of `T` that corresponds to the enum variant. For example, if you have a `struct T { name: String }` and `enum F { Name }`, you'll need to implement `PartialOrdBy<T>` for `F` like this:

```rust
impl PartialOrdBy<T> for F {
    fn partial_cmp_by(&self, a: &T, b: &T) -> Option<Ordering> {
        match self {
            F::Name => a.name.partial_cmp(&b.name),
        }
    }
}
```

The second trait `Sortable` implements a `sort_by` method that describes how the field may be sorted. Carrying on with our example, we'll implement `Sortable` for `F` like this:

```rust
impl Sortable for F {
    fn sort_by(&self) -> Option<SortBy> {
        match self {
            F::Name => SortBy::increasing_or_decreasing(),
        }
    }
}
```

The `increasing_or_decreasing` method returns `Some(SortBy)` if the field can be sorted in both directions. If it can only be sorted in one direction, use `increasing` or `decreasing` instead. You can also specify `decreasing_or_increasing` if you want the default sort order to be decreasing but still allow both.

This trait also has a provided method `null_handling` that specifies how `NULL` values are treated. These occur if `PartialOrdBy<T>` returns `None` for non-comparable values. You have the option of ordering `NULL` values first or last. The default is to order them last.

In combination both traits offer semantics similar to SQL's [ORDER BY clause](https://www.postgresql.org/docs/current/sql-select.html#SQL-ORDERBY): `ORDER BY expression [ ASC | DESC | USING operator ] [ NULLS { FIRST | LAST } ] [, ...]`

Finally we build a compoent to render a sortable table. The `th` cells are clickable and will sort the table by the corresponding field.

```rust
#[inline_props]
pub fn TableOfT(cx: Scope) -> Element {
    // Load data from somewhere
    let mut data = load_data();

    // Create our sorter
    let sorter = use_sorter::<MyStructField>(cx);
    sorter.sort(data.as_mut_slice());

    // Render a table with click-to-sort headers
    cx.render(rsx! {
        table {
            thead {
                tr {
                    Th { sorter: sorter, field: F::Name, rsx!("Name") }
                }
            }
            tbody {
                data.iter().map(|row| {
                    rsx! {
                        tr {
                            td { "{row.name}" }
                        }
                    }
                })
            }
        }
    })
}
```

## Using the library

Run a minimal example: `dioxus serve --example table`

Generate docs: `cargo doc --open --no-deps`

Feedback welcome.

## TODO

- More testing.
- Configurable rsx helpers.
- Create a UseSorter::set_field (that also checks the direction is valid).
- UseSorterBuilder should rely on set_field and simplify its logic.
