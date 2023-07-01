use dioxus::prelude::*;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sorter<'a, F: 'static> {
    field: &'a UseState<F>,
    direction: &'a UseState<Direction>,
}

pub trait PartialOrdBy<T>: PartialEq {
    fn partial_cmp_by(&self, a: &T, b: &T) -> Option<Ordering>;
}

pub trait Sortable: PartialEq {
    fn sort_by(&self) -> Option<SortBy>;
    fn null_handling(&self) -> NullHandling {
        NullHandling::Last
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SortBy {
    Fixed(Direction),
    Reversible(Direction),
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Direction {
    #[default]
    Ascending,
    Descending,
}

impl Direction {
    pub fn invert(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum NullHandling {
    First,
    #[default]
    Last,
}

impl Default for SortBy {
    fn default() -> SortBy {
        Self::increasing_or_decreasing().unwrap()
    }
}

impl SortBy {
    pub fn unsortable() -> Option<Self> {
        None
    }
    pub fn increasing() -> Option<Self> {
        Some(Self::Fixed(Direction::Ascending))
    }
    pub fn decreasing() -> Option<Self> {
        Some(Self::Fixed(Direction::Descending))
    }
    pub fn increasing_or_decreasing() -> Option<Self> {
        Some(Self::Reversible(Direction::Ascending))
    }
    pub fn decreasing_or_increasing() -> Option<Self> {
        Some(Self::Reversible(Direction::Descending))
    }

    pub fn direction(&self) -> Direction {
        match self {
            Self::Fixed(dir) => *dir,
            Self::Reversible(dir) => *dir,
        }
    }
}

// TODO add builder to set initial sort params + configurable form

pub fn use_sorter<F: Sortable + Default>(cx: &ScopeState) -> Sorter<'_, F> {
    let field = F::default();
    let dir = field.sort_by().unwrap_or_default().direction();
    Sorter {
        field: use_state(cx, || field),
        direction: use_state(cx, || dir),
    }
}

impl<'a, F> Sorter<'a, F> {
    pub fn get_state(&self) -> (&F, &Direction) {
        (self.field.get(), self.direction.get())
    }

    pub fn set_field(&self, field: F)
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
