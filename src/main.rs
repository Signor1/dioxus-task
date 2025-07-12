use dioxus::prelude::*;
use rusqlite::Connection;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let con = use_signal(|| {
        let conn = Connection::open("./data.db3").unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
            (),
        )
        .unwrap();
        conn
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Main { connect: con }
    }
}

#[component]
pub fn Main(connect: Signal<Connection>) -> Element {
    let mut item = use_signal(|| String::new());

    let mut items = use_signal(Vec::<Item>::new);

    let mut add_item = move |item_name: String| {
        if item_name.trim().is_empty() {
            return;
        }
        connect
            .write()
            .execute("INSERT INTO app (name) VALUES (?1)", [&item_name])
            .unwrap();
    };

    let delete_item = move |item: Item| {
        connect
            .write()
            .execute("DELETE FROM app WHERE id = ?1", [&item.id])
            .unwrap();
    };

    use_effect(move || {
        items.write().clear();

        let read = connect.read();
        let mut stm = read.prepare("SELECT * FROM app").unwrap();

        let rows = stm
            .query_map((), |row| {
                Ok(Item {
                    id: row.get(0).unwrap(),
                    name: row.get(1).unwrap(),
                })
            })
            .unwrap();

        for row in rows {
            let item = row.unwrap();
            items.write().push(item);
        }
    });

    rsx! {
        div { class: "container",
            div { class: "img-wrapper",
                img {
                    src: HEADER_SVG,
                    alt: "Header Image",
                    class: "header-image",
                }
                h1 { "A TodoList MacOS App built with Dioxus(Rust)" }
            }
            div { class: "header",
                label { "Enter a task" }
                input {
                    r#type: "text",
                    class: "input",
                    placeholder: "I want to...",
                    value: "{item}",
                    oninput: move |event| { item.set(event.value()) },
                    onkeydown: move |event| {
                        if event.key() == Key::Enter {
                            add_item(item());
                            item.set(String::new());
                        }
                    },
                }
            }
            div { class: "todos",
                if items.is_empty() {
                    p { "No tasks available. Add a task to get started!" }
                } else {
                    p { "List of tasks" }
                    for item in items.iter() {
                        TodoItem { item: item.clone(), callback: delete_item }
                    }
                }
            }
        }
    }
}

#[component]
fn TodoItem(item: Item, callback: Callback<Item>) -> Element {
    rsx! {
        div { class: "todo-item",
            p { {item.name.clone()} }
            button {
                class: "delete-button",
                onclick: move |_event| {
                    callback(item.clone());
                },
                "Delete"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Item {
    id: u32,
    name: String,
}
