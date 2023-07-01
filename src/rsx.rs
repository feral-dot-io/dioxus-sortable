#![allow(non_snake_case)]
use crate::{Direction, SortBy, Sortable, UseSorter};
use dioxus::prelude::*;

#[doc(hidden)]
#[derive(Props)]
pub struct ThProps<'a, F: 'static> {
    sorter: UseSorter<'a, F>,
    field: F,
    children: Element<'a>,
}

pub fn Th<'a, F: Copy + Sortable>(cx: Scope<'a, ThProps<'a, F>>) -> Element<'a> {
    let sorter = cx.props.sorter;
    let field = cx.props.field;
    cx.render(rsx! {
        th {
            onclick: move |_| sorter.toggle_field(field),
            &cx.props.children
            ThStatus {
                sorter: sorter,
                field: field,
            }
        }
    })
}

#[doc(hidden)]
#[derive(PartialEq, Props)]
pub struct ThStatusProps<'a, F: 'static> {
    sorter: UseSorter<'a, F>,
    field: F,
}

pub fn ThStatus<'a, F: Copy + Sortable>(cx: Scope<'a, ThStatusProps<'a, F>>) -> Element<'a> {
    let sorter = &cx.props.sorter;
    let field = cx.props.field;
    let (active_field, active_dir) = sorter.get_state();
    let active = *active_field == field;

    cx.render(match field.sort_by() {
        None => rsx!(""),
        Some(sort_by) => {
            use Direction::*;
            use SortBy::*;
            match sort_by {
                Fixed(Ascending) => rsx!(ThSpan { active: active, "↓" }),
                Fixed(Descending) => rsx!(ThSpan { active: active, "↑" }),

                Reversible(_) => rsx!(
                ThSpan {
                    active: active,
                    match (active, active_dir) {
                        (true, Direction::Ascending) => "↓",
                        (true, Direction::Descending) => "↑",
                        (false, _) => "↕",
                    }
                }),
            }
        }
    })
}

#[derive(Props)]
struct ThSpan<'a> {
    active: bool,
    children: Element<'a>,
}

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
