use dioxus::prelude::*;
use dioxus_sortable::{
    Direction, NullHandling, PartialOrdBy, SortBy, Sortable, Th, UseSorterBuilder,
};
use std::cmp::Ordering;

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    dioxus_web::launch(view);
}

pub fn view(cx: Scope) -> Element {
    let mut data = load_data();

    let sorter = UseSorterBuilder::default()
        .with_field(MyStructField::Second)
        .with_direction(Direction::Descending)
        .use_sorter(cx);
    //let sorter = use_sorter::<MyStructField>(cx);
    sorter.sort(data.as_mut_slice());

    cx.render(rsx! {
        table {
            thead {
                tr {
                    Th { sorter: sorter, field: MyStructField::First, rsx!("First") }
                    Th { sorter: sorter, field: MyStructField::Second, rsx!("Second") }
                    Th { sorter: sorter, field: MyStructField::Third, rsx!("Third") }
                    Th { sorter: sorter, field: MyStructField::Fourth, rsx!("Fourth") }
                }
            }
            tbody {
                data.iter().map(|row| {
                    fn fmt_f64(f: Option<f64>) -> String {
                        f.map_or_else(|| "-".to_string(), |x| x.to_string())
                    }

                    rsx! {
                        tr {
                            td { "{row.first}" }
                            td { "{row.second}" }
                            td { "{fmt_f64(row.third)}" }
                            td { "{fmt_f64(row.fourth)}" }
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
    fourth: Option<f64>,
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
            fourth: if i % 4 != 2 {
                Some((i * 10) as f64)
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
    Fourth,
}

impl PartialOrdBy<MyStruct> for MyStructField {
    fn partial_cmp_by(&self, a: &MyStruct, b: &MyStruct) -> Option<Ordering> {
        fn cmp_f64(a: Option<f64>, b: Option<f64>) -> Option<Ordering> {
            a.unwrap_or(f64::NAN).partial_cmp(&b.unwrap_or(f64::NAN))
        }

        use MyStructField::*;
        match self {
            First => a.first.partial_cmp(&b.first),
            Second => a.second.partial_cmp(&b.second),
            Third => cmp_f64(a.third, b.third),
            Fourth => cmp_f64(a.fourth, b.fourth),
        }
    }
}

impl Sortable for MyStructField {
    fn sort_by(&self) -> Option<SortBy> {
        match self {
            MyStructField::First => SortBy::increasing(),
            MyStructField::Second => SortBy::increasing_or_decreasing(),
            MyStructField::Third => SortBy::increasing_or_decreasing(),
            MyStructField::Fourth => SortBy::decreasing_or_increasing(),
        }
    }

    fn null_handling(&self) -> NullHandling {
        match self {
            MyStructField::Fourth => NullHandling::First,
            _ => NullHandling::default(),
        }
    }
}
