use dioxus::prelude::*;
use dioxus_sortable::{use_sorter, NullHandling, PartialOrdBy, SortBy, Sortable, Th, ThStatus};

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    // Trigger pulling our data "externally"
    let future = use_future(cx, (), |_| load_prime_ministers());

    cx.render(rsx! {
        h1 { "Birthplaces of British prime ministers" }
        future.value().map_or_else(
            // Show a loading message while the data is being fetched
            || rsx!{
                p { "Loading..." }
            },
            // Pass the data onto our table component
            |data| rsx!{
                PrimeMinisters{ data: data.to_vec(), }
            })
    })
}

/// Creates a sortable table of prime ministers and their birthplaces. Can be filtered by name.
///
/// Each column header can be clicked to sort by that column. The current sort state is displayed in the header.
#[allow(non_snake_case)]
#[inline_props]
fn PrimeMinisters(cx: Scope, data: Vec<Person>) -> Element {
    // Sorter hook must be called unconditionally
    let sorter = use_sorter::<PersonField>(cx);
    let name = use_state(cx, || "".to_string());

    // Filter the data
    let mut data = data
        .to_owned()
        .into_iter()
        .filter(|row| row.name.to_lowercase().contains(&name.get().to_lowercase()))
        .collect::<Vec<_>>();
    // Sort the data. Unlike use_sorter, may be skipped
    sorter.sort(data.as_mut_slice());

    cx.render(rsx! {
        // Our simple search box
        input {
            placeholder: "Search by name",
            oninput: move |evt| name.set(evt.value.clone()),
        }

        // Render a table like we would any other except for the `Th` component
        table {
            thead {
                tr {
                    // The `Th` helper component is used to render a sortable column header
                    Th { sorter: sorter, field: PersonField::Name, "Name" }
                    // It will display an arrow to indicate the current sort direction and state
                    Th { sorter: sorter, field: PersonField::LeftOffice, "Left office" }
                    // Here's how we might do it manually:
                    th {
                        // The `toggle_field` method triggers the state change and sorts the table
                        onclick: move |_| sorter.toggle_field(PersonField::Birthplace),
                        "Birthplace"
                        // The `ThStatus` helper component renders the current state. You could write your own custom status viewer with by using [`UseSorter::get_state`].
                        ThStatus {
                            sorter: sorter,
                            field: PersonField::Birthplace,
                        }
                    }
                    // We could also skip all the status fields entirely and have a separate form for controlling the sort state. In that situation we'd use the normal td {} elements.
                    Th { sorter: sorter, field: PersonField::Country, "Country" }
                }
            }
            tbody {
                // Iterate over our Person data like we would any other.
                data.iter().map(|row| {
                    rsx! {
                        tr {
                            td { "{row.name}" }
                            td {
                                match row.left_office {
                                    None => rsx!(em { "Present" }),
                                    Some(ref x) => rsx!("{x}"),
                                }
                            }
                            td {
                                match row.birthplace {
                                    Birthplace::Unknown => rsx!(em { "Unknown" }),
                                    Birthplace::City(ref city) => rsx!("{city}")
                                }
                            }
                            td { "{row.country}" }
                        }
                    }
                })
            }
        }
    })
}

/// Our per-row data type that we want to sort
#[derive(Clone, Debug, PartialEq)]
struct Person {
    name: String,
    /// None means the person is still in office
    left_office: Option<u32>,
    /// Another way of dealing with unknown values is to use an enum
    birthplace: Birthplace,
    country: String,
}

/// Where was the person born?
#[derive(Clone, Debug, Default, PartialEq)]
enum Birthplace {
    #[default]
    Unknown,
    City(String),
}

/// This is the field we want to sort by. Each variant corresponds to a column in our table or field in our Person struct. Keep it simple, use `{struct}Field`.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
enum PersonField {
    Name,
    #[default]
    LeftOffice,
    Birthplace,
    Country,
}

/// This trait decides how our rows are sorted
impl PartialOrdBy<Person> for PersonField {
    fn partial_cmp_by(&self, a: &Person, b: &Person) -> Option<std::cmp::Ordering> {
        match self {
            // Most values like Strings, integers and f64 require no special treatment and partial_cmp can be used directly.
            PersonField::Name => a.name.partial_cmp(&b.name),

            // We're using None to say that the person is still in office which becomes our NULL value. There's a gotcha here as Rust will treat None values as smaller than Some (not NULL).
            PersonField::LeftOffice => a
                .left_office
                .zip(b.left_office)
                .and_then(|(a, b)| a.partial_cmp(&b)),

            // Similar to LeftOffice, our unknown birthplace is treated as NULL value. Instead of using Option we've wrapped it up in our own enum closer to what we'd do in a real app. If either of the values are unknown (our NULL) then we must return None. Otherwise do a standard comparison on the city name.
            PersonField::Birthplace => match (&a.birthplace, &b.birthplace) {
                (Birthplace::Unknown, _) => None,
                (_, Birthplace::Unknown) => None,
                (Birthplace::City(a), Birthplace::City(b)) => a.partial_cmp(b),
            },

            PersonField::Country => a.country.partial_cmp(&b.country),
        }
    }
}

/// This trait decides how fields (columns) may be sorted
impl Sortable for PersonField {
    fn sort_by(&self) -> Option<SortBy> {
        use PersonField::*;
        match self {
            Name => SortBy::increasing_or_decreasing(),
            // Let's say we want our list to focus on the most recent prime ministers. We can use decreasing sort order to put the most recent ones at the top. Another useful value would be decreasing_or_increasing which would allow the user to toggle the column.
            LeftOffice => SortBy::decreasing(),
            // You might notice increasing_or_decreasing is used a lot. It's the default value.
            Birthplace => SortBy::increasing_or_decreasing(),
            // An acceptable implementation of sort_by is to return increasing_or_decreasing for all fields.
            Country => SortBy::increasing_or_decreasing(),
        }
    }

    fn null_handling(&self) -> NullHandling {
        use PersonField::*;
        match self {
            // Our left office column is ordered from most recent to oldest. We want the most recent prime minister to be at the top of the list. So since we're using the NULL value to mean they're still in office, we want NULLs to come first.
            LeftOffice => NullHandling::First,

            // We don't normally have to specify null_handling. The default value is NullHandling::Last.
            _ => NullHandling::Last,
        }
    }
}

impl Person {
    /// Helper function for load_prime_ministers to create a new Person
    fn new(
        name: &'static str,
        left: impl Into<Option<u32>>,
        birth: impl Into<Option<&'static str>>,
        country: &'static str,
    ) -> Person {
        Person {
            name: name.to_string(),
            left_office: left.into(),
            birthplace: match birth.into() {
                Some(city) => Birthplace::City(city.to_string()),
                None => Birthplace::Unknown,
            },
            country: country.to_string(),
        }
    }
}

/// Our mock data source. In a real app this could be something like a `reqwest` call
async fn load_prime_ministers() -> Vec<Person> {
    vec![
        Person::new("Robert Walpole", 1742, "Houghton Hall, Norfolk", "England"),
        Person::new(
            "Spencer Compton",
            1743,
            "Compton Wynyates, Warwickshire",
            "England",
        ),
        Person::new("Henry Pelham", 1756, "Laughton, Sussex", "England"),
        Person::new("William Cavendish", 1757, None, "England"),
        Person::new("Thomas Pelham-Holles", 1762, "London", "England"),
        Person::new(
            "John Stuart",
            1763,
            "Parliament Square, Edinburgh",
            "Scotland",
        ),
        Person::new(
            "George Grenville",
            1765,
            "Wotton, Buckinghamshire",
            "England",
        ),
        Person::new("William Pitt", 1768, "Westminster, London", "England"),
        Person::new("Augustus FitzRoy", 1770, None, "England"),
        Person::new("Frederick North", 1782, "Piccadilly, London", "England"),
        Person::new(
            "Charles Watson-Wentworth",
            1782,
            "Wentworth, Yorkshire",
            "England",
        ),
        Person::new(
            "William Petty",
            1783,
            "Dublin, County Dublin",
            "Republic of Ireland",
        ),
        Person::new("Henry Addington", 1804, "Holborn, London", "England"),
        Person::new("William Pitt", 1806, "Hayes, Kent", "England"),
        Person::new(
            "William Grenville",
            1807,
            "Wotton, Buckinghamshire",
            "England",
        ),
        Person::new(
            "William Cavendish-Bentinck",
            1809,
            "Nottinghamshire",
            "England",
        ),
        Person::new("Spencer Perceval", 1812, "Mayfair, London", "England"),
        Person::new("Robert Jenkinson", 1827, "London", "England"),
        Person::new("George Canning", 1827, "Marylebone, London", "England"),
        Person::new(
            "Frederick Robinson",
            1828,
            "Skelton-on-Ure, Yorkshire",
            "England",
        ),
        Person::new(
            "Arthur Wellesley",
            1834,
            "Dublin, County Dublin",
            "Republic of Ireland",
        ),
        Person::new("William Lamb", 1841, "London", "England"),
        Person::new("Robert Peel", 1846, "Bury, Lancashire", "England"),
        Person::new(
            "George Hamilton Gordon",
            1855,
            "Edinburgh, Midlothian",
            "Scotland",
        ),
        Person::new(
            "Henry John Temple, Lord Palmerston",
            1858,
            "Westminster, Middlesex",
            "England",
        ),
        Person::new("John Russell", 1852, "Mayfair, Middlesex", "England"),
        Person::new(
            "Edward Smith-Stanley",
            1852,
            "Knowsley Hall, Knowsley, Lancashire",
            "England",
        ),
        Person::new(
            "Benjamin Disraeli",
            1868,
            "Bloomsbury, Middlesex",
            "England",
        ),
        Person::new(
            "William Ewart Gladstone",
            1894,
            "Liverpool, Lancashire",
            "England",
        ),
        Person::new("Archibald Primrose", 1895, "Mayfair, Middlesex", "England"),
        Person::new(
            "Robert Gascoyne-Cecil",
            1902,
            "Hatfield, Hertfordshire",
            "England",
        ),
        Person::new(
            "Arthur Balfour",
            1905,
            "Whittingehame, East Lothian",
            "Scotland",
        ),
        Person::new(
            "Henry Campbell-Bannerman",
            1908,
            "Kelvinside, Glasgow",
            "Scotland",
        ),
        Person::new(
            "H. H. Asquith",
            1916,
            "Morley, West Riding of Yorkshire",
            "England",
        ),
        Person::new(
            "David Lloyd George",
            1922,
            "Chorlton-on-Medlock, Lancashire,",
            "England",
        ),
        Person::new("Bonar Law", 1923, "Rexton, Kent County", "Canada"),
        Person::new(
            "Ramsay MacDonald",
            1935,
            "Lossiemouth, Morayshire",
            "Scotland",
        ),
        Person::new(
            "Stanley Baldwin",
            1937,
            "Bewdley, Worcestershire",
            "England",
        ),
        Person::new(
            "Neville Chamberlain",
            1940,
            "Edgbaston, Birmingham",
            "England",
        ),
        Person::new(
            "Winston Churchill",
            1955,
            "Blenheim, Oxfordshire",
            "England",
        ),
        Person::new("Clement Attlee", 1951, "Putney, Surrey", "England"),
        Person::new(
            "Anthony Eden",
            1957,
            "Windlestone Hall, County Durham",
            "England",
        ),
        Person::new("Harold Macmillan", 1963, "Belgravia, London", "England"),
        Person::new("Alec Douglas-Home", 1964, "Mayfair, London", "England"),
        Person::new(
            "Harold Wilson",
            1976,
            "Huddersfield, West Riding of Yorkshire",
            "England",
        ),
        Person::new("Edward Heath", 1974, "Broadstairs, Kent", "England"),
        Person::new("James Callaghan", 1979, "Portsmouth, Hampshire", "England"),
        Person::new(
            "Margaret Thatcher",
            1990,
            "Grantham, Lincolnshire",
            "England",
        ),
        Person::new("John Major", 1997, "St Helier, Surrey", "England"),
        Person::new("Tony Blair", 2007, "Edinburgh, Midlothian", "Scotland"),
        Person::new("Gordon Brown", 2010, "Giffnock, Renfrewshire", "Scotland"),
        Person::new("David Cameron", 2016, "Marylebone, London", "England"),
        Person::new("Theresa May", 2019, "Eastbourne, East Sussex", "England"),
        Person::new(
            "Boris Johnson",
            2022,
            "New York City, New York",
            "United States",
        ),
        Person::new("Liz Truss", 2022, "Oxford, Oxfordshire", "England"),
        Person::new("Rishi Sunak", None, "Southampton, Hampshire", "England "),
    ]
}
