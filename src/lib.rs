use dioxus::prelude::*;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sorter<'a, F: 'static> {
    field: &'a UseState<F>,
    direction: &'a UseState<Direction>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
enum Direction {
    #[default]
    Ascending,
    Descending,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
enum NullHandling {
    First,
    #[default]
    Last,
}

impl Direction {
    pub fn invert(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

pub trait PartialOrdBy<T>: PartialEq {
    fn partial_cmp_by(&self, a: &T, b: &T) -> Option<Ordering>;
}

pub trait Sortable: PartialEq {
    fn sort_by(&self) -> SortBy;
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SortBy {
    options: SortOptions,
    nulls: NullHandling,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
enum SortOptions {
    Unsortable,
    Increasing,
    Decreasing,
    #[default]
    IncreasingOrDecreasing,
    DecreasingOrIncreasing,
}

impl SortBy {
    pub fn unsortable() -> Self {
        Self {
            options: SortOptions::Unsortable,
            nulls: NullHandling::default(),
        }
    }
    pub fn increasing() -> Self {
        Self {
            options: SortOptions::Increasing,
            nulls: NullHandling::default(),
        }
    }
    pub fn decreasing() -> Self {
        Self {
            options: SortOptions::Decreasing,
            nulls: NullHandling::default(),
        }
    }
    pub fn increasing_or_decreasing() -> Self {
        Self {
            options: SortOptions::IncreasingOrDecreasing,
            nulls: NullHandling::default(),
        }
    }
    pub fn decreasing_or_increasing() -> Self {
        Self {
            options: SortOptions::DecreasingOrIncreasing,
            nulls: NullHandling::default(),
        }
    }

    pub fn nulls_first(&self) -> Self {
        Self {
            nulls: NullHandling::First,
            ..*self
        }
    }
    pub fn nulls_last(&self) -> Self {
        Self {
            nulls: NullHandling::Last,
            ..*self
        }
    }
}

impl SortOptions {
    fn initial_direction(&self) -> Direction {
        use Direction::*;
        use SortOptions::*;
        match self {
            Unsortable => Direction::default(),
            Increasing => Ascending,
            Decreasing => Descending,
            IncreasingOrDecreasing => Ascending,
            DecreasingOrIncreasing => Descending,
        }
    }
}

// TODO add builder to set initial sort params + configurable form

pub fn use_sorter<F: Sortable + Default>(cx: &ScopeState) -> Sorter<'_, F> {
    let field = F::default();
    let initial_dir = field.sort_by().options.initial_direction();
    let sorter = use_state(cx, || field);
    let direction = use_state(cx, || initial_dir);
    Sorter {
        field: sorter,
        direction,
    }
}

#[derive(Props)]
pub struct ThProps<'a, F: 'static> {
    sorter: Sorter<'a, F>,
    field: F,
    children: Element<'a>,
}

#[allow(non_snake_case)]
pub fn Th<'a, F: Copy + Sortable>(cx: Scope<'a, ThProps<'a, F>>) -> Element<'a> {
    let sorter = cx.props.sorter;
    let field = cx.props.field;
    cx.render(rsx! {
        th {
            onclick: move |_| sorter.set_field(field),
            &cx.props.children
            ThStatus {
                sorter: sorter,
                field: field,
            }
        }
    })
}

#[derive(PartialEq, Props)]
pub struct ThStatusProps<'a, F: 'static> {
    sorter: Sorter<'a, F>,
    field: F,
}

#[allow(non_snake_case)]
pub fn ThStatus<'a, F: Copy + Sortable>(cx: Scope<'a, ThStatusProps<'a, F>>) -> Element<'a> {
    let sorter = &cx.props.sorter;
    let field = cx.props.field;
    let active = *sorter.field.get() == field;
    let active_dir = *sorter.direction.get();

    use SortOptions::*;
    cx.render(match field.sort_by().options {
        Unsortable => rsx!(""),
        Increasing => rsx!(ThSpan { active: active, "↓" }),
        Decreasing => rsx!(ThSpan { active: active, "↑" }),

        IncreasingOrDecreasing | DecreasingOrIncreasing => rsx!(
        ThSpan {
            active: active,
            match (active, active_dir) {
                (true, Direction::Ascending) => "↓",
                (true, Direction::Descending) => "↑",
                (false, _) => "↕",
            }
        }),
    })
}

#[derive(Props)]
struct ThSpan<'a> {
    active: bool,
    children: Element<'a>,
}

#[allow(non_snake_case)]
fn ThSpan<'a>(cx: Scope<'a, ThSpan<'a>>) -> Element<'a> {
    let colour = if cx.props.active { "#555" } else { "#ccc" };
    let nbsp = "&nbsp;";
    cx.render(rsx! {
        span {
            style: "color: {colour};",
            span { dangerous_inner_html: "{nbsp}", }
            &cx.props.children
        }
    })
}

impl<'a, F> Sorter<'a, F> {
    #[allow(non_snake_case)]
    pub fn Th(&self, cx: &'a ScopeState, field: F, children: LazyNodes<'a, 'a>) -> Element<'a>
    where
        F: Copy + Sortable,
    {
        cx.render(rsx! {
            Th {
                sorter: *self,
                field: field,
                children
            }
        })
    }

    pub fn set_field(&self, new: F)
    where
        F: Sortable,
    {
        let sort_by = new.sort_by().options;
        // Don't change if unsortable
        if sort_by == SortOptions::Unsortable {
            return;
        }
        // Same field, invert direction
        if *self.field.get() == new {
            // Invert direction if both directions are allowed
            use SortOptions::*;
            if sort_by == IncreasingOrDecreasing || sort_by == DecreasingOrIncreasing {
                self.direction.modify(|v| v.invert());
            }
        } else {
            // Otherwise set new field and direction
            self.field.set(new);
            self.direction.set(sort_by.initial_direction());
        }
    }

    pub fn sort<T>(&self, items: &mut [T])
    where
        F: PartialOrdBy<T> + Sortable,
    {
        let nulls = self.field.sort_by().nulls;
        sort_by(self.field.get(), *self.direction.get(), nulls, items);
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
