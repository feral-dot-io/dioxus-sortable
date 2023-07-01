use dioxus::prelude::*;
use dioxus_sortable::{use_sorter, PartialOrdBy, SortBy, Sortable, Sorter};
use std::cmp::Ordering;

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    dioxus_web::launch(view);
}

pub fn view(cx: Scope) -> Element {
    let mut data = load_data();

    let sorter = use_sorter::<MyStructField>(cx);
    sorter.sort(data.as_mut_slice());

    cx.render(rsx! {
        table {
            thead {
                tr {
                    sorter.Th(cx, MyStructField::First, rsx!("First") )
                    sorter.Th(cx, MyStructField::Second, rsx!("Second") )
                    sorter.Th(cx, MyStructField::Third, rsx!("Third") )
                }
            }
            tbody {
                data.iter().map(|row| {
                    let third = row.third.map_or_else(|| "-".to_string(), |x| x.to_string());
                    rsx! {
                        tr {
                            td { "{row.first}" }
                            td { "{row.second}" }
                            td { "{third}" }
                        }
                    }
                })
            }
        }
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct MyStruct {
    first: String,
    second: i32,
    third: Option<f64>,
}

fn load_data() -> Vec<MyStruct> {
    (0..10)
        .map(|i| MyStruct {
            first: format!("#{i}"),
            second: i * 2,
            third: if i % 4 != 3 {
                Some((10 - i) as f64 + (i as f64) / 10.0)
            } else {
                None
            },
        })
        .collect()
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum MyStructField {
    First,
    #[default]
    Second,
    Third,
}

impl PartialOrdBy<MyStruct> for MyStructField {
    fn partial_cmp_by(&self, a: &MyStruct, b: &MyStruct) -> Option<Ordering> {
        use MyStructField::*;
        match self {
            First => a.first.partial_cmp(&b.first),
            Second => a.second.partial_cmp(&b.second),
            Third => a
                .third
                .unwrap_or(f64::NAN)
                .partial_cmp(&b.third.unwrap_or(f64::NAN)),
        }
    }
}

impl Sortable for MyStructField {
    fn sort_by(&self) -> SortBy {
        match self {
            MyStructField::First => SortBy::increasing(),
            MyStructField::Second => SortBy::increasing_or_decreasing(),
            MyStructField::Third => SortBy::decreasing_or_increasing().nulls_first(),
        }
    }
}
