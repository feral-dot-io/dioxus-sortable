use dioxus::prelude::*;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sorter<'a, F: 'static> {
    field: &'a UseState<F>,
    direction: &'a UseState<Direction>,
    nulls: NullHandling,
}

pub trait PartialOrdBy<T> {
    fn partial_cmp_by(&self, a: &T, b: &T) -> Option<Ordering>;

    fn sortable(&self) -> Sortable {
        Sortable::default()
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Sortable {
    Unsortable,
    Increasing,
    Decreasing,
    #[default]
    IncreasingOrDecreasing,
    DecreasingOrIncreasing,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Direction {
    #[default]
    Ascending,
    Descending,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum NullHandling {
    First,
    #[default]
    Last,
}

pub fn use_sorter<F: Default>(cx: &ScopeState) -> Sorter<'_, F> {
    use_sorter_with(
        cx,
        F::default(),
        Direction::default(),
        NullHandling::default(),
    )
}

pub fn use_sorter_with<T>(
    cx: &ScopeState,
    field: T,
    direction: Direction,
    nulls: NullHandling,
) -> Sorter<'_, T> {
    let sorter = use_state(cx, || field);
    let direction = use_state(cx, || direction);
    Sorter {
        field: sorter,
        direction,
        nulls,
    }
}

#[derive(Props)]
pub struct SortableThProps<'a, F: 'static> {
    sorter: Sorter<'a, F>,
    field: F,
    children: Element<'a>,
}

#[allow(non_snake_case)]
pub fn Th<'a, F, T>(cx: Scope<'a, SortableThProps<'a, F>>) -> Element<'a>
where
    F: Copy + PartialEq + PartialOrdBy<T>,
{
    let sorter = cx.props.sorter;
    let field = cx.props.field;
    cx.render(rsx! {
        th {
            onclick: move |_| sorter.set(field),
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
pub fn ThStatus<'a, T, F>(cx: Scope<'a, ThStatusProps<'a, F>>) -> Element<'a>
where
    F: Copy + PartialEq + PartialOrdBy<T>,
{
    let sorter = &cx.props.sorter;
    let field = cx.props.field;
    let active = *sorter.field.get() == field;
    let active_dir = *sorter.direction.get();

    use Direction::*;
    use Sortable::*;
    cx.render(match field.sortable() {
        Unsortable => rsx!(""),
        Increasing => rsx!(ThSpan { active: active, "↓" }),
        Decreasing => rsx!(ThSpan { active: active, "↑" }),
        IncreasingOrDecreasing | DecreasingOrIncreasing => rsx!(
        ThSpan {
            active: active,
            match (active, active_dir) {
                (true, Ascending) => "↓",
                (true, Descending) => "↑",
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

impl Direction {
    pub fn invert(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

impl<'a, F> Sorter<'a, F> {
    #[allow(non_snake_case)]
    pub fn Th<T>(&self, cx: &'a ScopeState, field: F, children: LazyNodes<'a, 'a>) -> Element<'a>
    where
        F: Copy + PartialEq + PartialOrdBy<T>,
    {
        cx.render(rsx! {
            Th {
                sorter: *self,
                field: field,
                children
            }
        })
    }

    pub fn set<T>(&self, new: F)
    where
        F: PartialOrdBy<T> + PartialEq,
    {
        use Sortable::*;
        let sortable = new.sortable();
        // Unsortable field, do nothing
        if sortable == Unsortable {
            return;
        }
        // Same field, invert direction
        if *self.field.get() == new {
            // Invert direction if both directions are allowed
            if sortable == IncreasingOrDecreasing || sortable == DecreasingOrIncreasing {
                self.direction.modify(|v| v.invert());
            }
        } else {
            // Otherwise set new field and direction
            self.field.set(new);
            self.direction.set(match sortable {
                Unsortable => unreachable!(),
                Increasing | IncreasingOrDecreasing => Direction::Ascending,
                Decreasing | DecreasingOrIncreasing => Direction::Descending,
            });
        }
    }

    pub fn sort<T>(&self, items: &mut [T])
    where
        F: PartialOrdBy<T> + PartialEq,
    {
        sort_by(self.field.get(), *self.direction.get(), self.nulls, items);
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
