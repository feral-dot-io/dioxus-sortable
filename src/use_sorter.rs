use dioxus::prelude::*;
use std::cmp::Ordering;

/// Stores Dioxus hooks and state of our sortable items.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UseSorter<'a, F: 'static> {
    field: &'a UseState<F>,
    direction: &'a UseState<Direction>,
}

/// Trait used by [UseSorter](UseSorter) to sort a struct by a specific field. This must be implemented on the field enum. Type `T` represents the struct (table row) that is being sorted.
///
/// The implementation should use the [`PartialOrd::partial_cmp`] trait to compare the field values and return the result. For example:
/// ```rust
/// # use dioxus_sortable::PartialOrdBy;
/// # use std::cmp::Ordering;
/// # #[derive(PartialEq)]
/// struct MyStruct {
///     first: String,
///     second: f64, // <- Note: can return None if f64::NAN
/// }
///
/// # #[derive(Copy, Clone, Debug, PartialEq)]
/// enum MyStructField {
///     First,
///     Second,
/// }
///
/// impl PartialOrdBy<MyStruct> for MyStructField {
///     fn partial_cmp_by(&self, a: &MyStruct, b: &MyStruct) -> Option<Ordering> {
///         match self {
///             MyStructField::First => a.first.partial_cmp(&b.first),
///             MyStructField::Second => a.second.partial_cmp(&b.second),
///         }
///     }
/// }
/// ```
pub trait PartialOrdBy<T>: PartialEq {
    /// Compare two values of type `T` by the field's enum. Return values of `None` are treated as `NULL` values. See [`Sortable`] for more information.
    ///
    /// Be careful when comparing types like `Option` which implement `Ord`. This means that `None` and `Some` have an order where we might use them as unknown / `NULL` values. This can be a surprise.
    ///
    /// Another issue is `f64` only implements `PartialOrd` and not `Ord` because a value can hold `f64::NAN`. In this situation `partial_cmp` will return `None` and we'll treat these values as `NULL` as expected.
    fn partial_cmp_by(&self, a: &T, b: &T) -> Option<Ordering>;
}

/// Trait used to describe how a field can be sorted. This must be implemented on the field enum.
///
/// Our [`PartialOrdBy`] fn may result in `None` values which we refer to as `NULL`. We borrow from SQL here to handle these values in a similar way to the [SQL ORDER BY clause](https://www.postgresql.org/docs/current/sql-select.html#SQL-ORDERBY). The PostgreSQL general form is `ORDER BY expression [ ASC | DESC | USING operator ] [ NULLS { FIRST | LAST } ] [, ...]` where:
/// - `expression` is the field being sorted.
/// - `ASC` and `DESC` are the sort [`Direction`].
/// - `USING operator` is implied by [`PartialOrdBy`].
/// - `NULLS { FIRST | LAST }` corresponds to [`NullHandling`].
/// Meaning you can sort by ascending or descending and optionally specify `NULL` ordering.
pub trait Sortable: PartialEq {
    /// Describes how this field can be sorted.
    fn sort_by(&self) -> Option<SortBy>;

    /// Describes how `NULL` values (when [`PartialOrdBy`] returns `None`) should be ordered when sorting. Either all at the start or the end.
    ///
    /// Provided implementation relies on the default (all at the end) and should be overridden if you want to change this generally or on a per-field basis.
    fn null_handling(&self) -> NullHandling {
        NullHandling::default()
    }
}

/// Describes how a field should be sorted. Returned by [`Sortable::sort_by`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SortBy {
    /// This field is limited to being sorted in the one direction specified.
    Fixed(Direction),
    /// This field can be sorted in either direction. The direction specifies the initial direction. Fields of this sort can be toggled between directions.
    Reversible(Direction),
}

/// Sort direction. Does not have a default -- implied by the field via [`SortBy`].
///
/// Actual sorting is done by [`PartialOrdBy`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    /// Ascending sort. A-Z, 0-9, little to big, etc.
    Ascending,
    /// Descending sort. Z-A, opposite of ascending.
    Descending,
}

impl Direction {
    /// Inverts the direction.
    pub fn invert(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }

    fn from_field<F: Sortable>(field: &F) -> Direction {
        field.sort_by().unwrap_or_default().direction()
    }
}

/// Describes how `NULL` values should be ordered when sorting. We refer to `None` values returned from [`PartialOrdBy::partial_cmp_by`] as `NULL`. Warning: Rust's `Option::None` is not strictly equivalent to SQL's `NULL` but we borrow from SQL terminology to handle them.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum NullHandling {
    /// Places all `NULL` values first.
    First,
    /// Places all `NULL` values last. The default.
    #[default]
    Last,
}

impl Default for SortBy {
    fn default() -> SortBy {
        Self::increasing_or_decreasing().unwrap()
    }
}

impl SortBy {
    /// Field may not be sorted. Convenience fn for specifying how a field may be sorted.
    pub fn unsortable() -> Option<Self> {
        None
    }
    /// Field may only be sorted in ascending order.
    pub fn increasing() -> Option<Self> {
        Some(Self::Fixed(Direction::Ascending))
    }
    /// Field may only be sorted in descending order.
    pub fn decreasing() -> Option<Self> {
        Some(Self::Fixed(Direction::Descending))
    }
    /// Field may be sorted in either direction. The initial direction is ascending. This is the default.
    pub fn increasing_or_decreasing() -> Option<Self> {
        Some(Self::Reversible(Direction::Ascending))
    }
    /// Field may be sorted in either direction. The initial direction is descending.
    pub fn decreasing_or_increasing() -> Option<Self> {
        Some(Self::Reversible(Direction::Descending))
    }

    /// Returns the initial / implied direction of the sort.
    pub fn direction(&self) -> Direction {
        match self {
            Self::Fixed(dir) => *dir,
            Self::Reversible(dir) => *dir,
        }
    }

    fn ensure_direction(&self, dir: Direction) -> Direction {
        use SortBy::*;
        match self {
            // Must match allowed
            Fixed(allowed) if *allowed == dir => dir,
            // Did not match allowed
            Fixed(allowed) => *allowed,
            // Any allowed
            Reversible(_) => dir,
        }
    }
}

/// Builder for [UseSorter](UseSorter). Use this to specify the field and direction of the sorter. For example by passing sort state from URL parameters.
///
/// Ordering of [`Self::with_field`] and [`Self::with_direction`] matters as the builder will ignore invalid combinations specified by the field's [`Sortable`]. This is to prevent the user from specifying a direction that is not allowed by the field.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UseSorterBuilder<F> {
    field: F,
    direction: Direction,
}

impl<F: Default + Sortable> Default for UseSorterBuilder<F> {
    fn default() -> Self {
        let field = F::default();
        let direction = Direction::from_field(&field);
        Self { field, direction }
    }
}

impl<F: Copy + Default + Sortable> UseSorterBuilder<F> {
    /// Optionally sets the initial field to sort by.
    pub fn with_field(&self, field: F) -> Self {
        Self { field, ..*self }
    }

    /// Optionally sets the initial direction to sort by.[`Direction::Ascending`] can be set.
    pub fn with_direction(&self, direction: Direction) -> Self {
        Self { direction, ..*self }
    }

    /// Creates Dioxus hooks to manage state. Must follow Dioxus hook rules and be called unconditionally in the same order as other hooks. See [use_sorter()] for simple usage.
    ///
    /// This fn (or [`Self::use_sorter`]) *must* be called or never used. See the docs on [`UseSorter::sort`] on using conditions.
    ///
    /// If the field or direction has not been set then the default values will be used.
    pub fn use_sorter(self, cx: &ScopeState) -> UseSorter<F> {
        let sorter = use_sorter(cx);
        sorter.set_field(self.field, self.direction);
        sorter
    }
}

/// Creates Dioxus hooks to manage state. Must follow Dioxus hook rules and be called unconditionally in the same order as other hooks. See [UseSorterBuilder](UseSorterBuilder) for more advanced usage.
///
/// This fn (or [`UseSorterBuilder::use_sorter`]) *must* be called or never used. See the docs on [`UseSorter::sort`] on using conditions.
///
/// Relies on `F::default()` for the initial value.
pub fn use_sorter<F: Copy + Default + Sortable>(cx: &ScopeState) -> UseSorter<'_, F> {
    let field = F::default();
    UseSorter {
        field: use_state(cx, || field),
        direction: use_state(cx, || Direction::from_field(&field)),
    }
}

impl<'a, F> UseSorter<'a, F> {
    /// Returns the current field and direction. Can be used to recreate state with [UseSorterBuilder](UseSorterBuilder).
    pub fn get_state(&self) -> (&F, &Direction) {
        (self.field.get(), self.direction.get())
    }

    /// Sets the sort field and toggles the direction (if applicable). Ignores unsortable fields.
    pub fn toggle_field(&self, field: F)
    where
        F: Sortable,
    {
        match field.sort_by() {
            None => (), // Do nothing, don't switch to unsortable
            Some(sort_by) => {
                use SortBy::*;
                match sort_by {
                    Fixed(dir) => self.direction.set(dir),
                    Reversible(dir) => {
                        // Invert direction if the same field
                        let dir = if *self.field.get() == field {
                            self.direction.get().invert()
                        } else {
                            // Reset state to new field
                            dir
                        };
                        self.direction.set(dir);
                    }
                }
                self.field.set(field);
            }
        }
    }

    /// Sets the sort field and direction state directly. Ignores unsortable fields. Ignores the direction if not valid for a field.
    pub fn set_field(&self, field: F, dir: Direction)
    where
        F: Sortable,
    {
        match field.sort_by() {
            None => (), // Do nothing, ignore unsortable
            Some(sort_by) => {
                // Set state but ensure direction is valid
                let dir = sort_by.ensure_direction(dir);
                self.field.set(field);
                self.direction.set(dir);
            }
        }
    }

    /// Sorts items according to the current field and direction.
    ///
    /// This is not a hook and may be called conditionally. For example:
    /// - If data is coming from a `use_future` then you can call this fn once it has completed.
    /// - If you need to apply a filter, do so before calling this fn.
    pub fn sort<T>(&self, items: &mut [T])
    where
        F: PartialOrdBy<T> + Sortable,
    {
        let (field, dir) = self.get_state();
        sort_by(field, *dir, field.null_handling(), items);
    }
}

fn sort_by<T, F: PartialOrdBy<T>>(
    sort_by: &F,
    dir: Direction,
    nulls: NullHandling,
    items: &mut [T],
) {
    items.sort_by(|a, b| {
        let partial = sort_by.partial_cmp_by(a, b);
        partial.map_or_else(
            || {
                let a_is_null = sort_by.partial_cmp_by(a, a).is_none();
                let b_is_null = sort_by.partial_cmp_by(b, b).is_none();
                match (a_is_null, b_is_null) {
                    (true, true) => Ordering::Equal,
                    (true, false) => match nulls {
                        NullHandling::First => Ordering::Less,
                        NullHandling::Last => Ordering::Greater,
                    },
                    (false, true) => match nulls {
                        NullHandling::First => Ordering::Greater,
                        NullHandling::Last => Ordering::Less,
                    },
                    // Uh-oh, first partial_cmp_by should not have returned None
                    (false, false) => unreachable!(),
                }
            },
            // Reversal must be applied per item to avoid ordering NULLs
            |o| match dir {
                Direction::Ascending => o,
                Direction::Descending => o.reverse(),
            },
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Default, PartialEq)]
    struct Row(f64);

    #[derive(Copy, Clone, Debug, Default, PartialEq)]
    enum RowField {
        #[default]
        Value,
    }

    impl PartialOrdBy<Row> for RowField {
        fn partial_cmp_by(&self, a: &Row, b: &Row) -> Option<Ordering> {
            match self {
                Self::Value => a.0.partial_cmp(&b.0),
            }
        }
    }

    #[test]
    fn test_sort_by() {
        use Direction::*;
        use NullHandling::*;
        use RowField::*;

        // Ascending
        let mut rows = vec![Row(2.0), Row(1.0), Row(3.0)];
        sort_by(&Value, Ascending, First, rows.as_mut_slice());
        assert_eq!(rows, vec![Row(1.0), Row(2.0), Row(3.0)]);
        // Descending
        sort_by(&Value, Descending, First, rows.as_mut_slice());
        assert_eq!(rows, vec![Row(3.0), Row(2.0), Row(1.0)]);

        // Nulls last, ascending
        let mut rows = vec![Row(f64::NAN), Row(f64::NAN), Row(2.0), Row(1.0), Row(3.0)];
        sort_by(&Value, Ascending, Last, rows.as_mut_slice());
        assert_eq!(rows[0], Row(1.0));
        assert_eq!(rows[1], Row(2.0));
        assert_eq!(rows[2], Row(3.0));
        assert!(rows[3].0.is_nan());
        assert!(rows[4].0.is_nan());
        // Nulls first, ascending
        sort_by(&Value, Ascending, First, rows.as_mut_slice());
        assert!(rows[0].0.is_nan());
        assert!(rows[1].0.is_nan());
        assert_eq!(rows[2], Row(1.0));
        assert_eq!(rows[3], Row(2.0));
        assert_eq!(rows[4], Row(3.0));

        // Nulls last, descending
        sort_by(&Value, Descending, Last, rows.as_mut_slice());
        assert_eq!(rows[0], Row(3.0));
        assert_eq!(rows[1], Row(2.0));
        assert_eq!(rows[2], Row(1.0));
        assert!(rows[3].0.is_nan());
        assert!(rows[4].0.is_nan());
        // Nulls first, descending
        sort_by(&Value, Descending, First, rows.as_mut_slice());
        assert!(rows[0].0.is_nan());
        assert!(rows[1].0.is_nan());
        assert_eq!(rows[2], Row(3.0));
        assert_eq!(rows[3], Row(2.0));
        assert_eq!(rows[4], Row(1.0));
    }
}
