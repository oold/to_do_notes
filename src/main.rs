extern crate chrono;
extern crate rusqlite;

use std::io::{stderr, stdin, stdout, Write};

use app_dirs::*;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, NO_PARAMS, params, Result};

const APP_INFO: AppInfo = AppInfo { name: "to_do_notes", author: "Oliver Old" };

struct ToDo {
    id: i32,
    title: String,
    content: String,
    created: DateTime<Utc>,
    done: bool,
}

impl ToDo {
    fn print(&self) {
        println!(
            "[{}]\n\
            Title: {}\n\
            Content: {}\n\
            Time created: {}\n\
            Done: {}",
            self.id, self.title, self.content, self.created, self.done
        );
    }
}

fn create_item(database: &Connection) {
    print_prompt("Enter a title: ");
    let result = read_line();
    if result.is_err() {
        print_error(result.unwrap_err());
        return;
    }
    let title = result.unwrap();
    print_prompt("Enter the to do note: ");
    let result = read_line();
    if result.is_err() {
        print_error(result.unwrap_err());
        return;
    }
    let content = result.unwrap();
    database.execute(
        "INSERT INTO to_do_notes (title, content, created, done) \
            VALUES (?1, ?2, ?3, ?4)",
        params![title, content, Utc::now(), false],
    ).unwrap();
}

fn list_items(database: &Connection) {
    let mut statement = database.prepare(
        "SELECT id, title, content, created, done FROM to_do_notes"
    ).unwrap();
    let notes = statement.query_map(
        NO_PARAMS,
        |row| {
            Ok(ToDo {
                id: row.get(0).unwrap(),
                title: row.get(1).unwrap(),
                content: row.get(2).unwrap(),
                created: row.get(3).unwrap(),
                done: row.get(4).unwrap(),
            })
        },
    ).unwrap();
    for item in notes {
        println!("---");
        item.unwrap().print();
        println!("---");
    }
}

fn mark_done(database: &Connection) {
    print_prompt("Enter the ID of the item to mark done/undone: ");
    let result = get_item_selection();
    if result.is_err() {
        print_error(result.unwrap_err());
        return;
    }
    let choice = result.unwrap();
    let mut statement = database.prepare("SELECT done FROM to_do_notes WHERE id = ?1").unwrap();
    let mut notes = statement.query_map(
        params![choice],
        |row| -> Result<bool, rusqlite::Error> { Ok(row.get(0).unwrap()) },
    ).unwrap();
    let first_note = notes.next();
    if first_note.is_none() {
        eprintln!("Chosen to do note does not exist.");
        return;
    }
    let done = first_note.unwrap().unwrap();
    let mut statement = database.prepare("UPDATE to_do_notes SET done = ?1 WHERE id = ?2").unwrap();
    statement.execute(params![!done, choice]).unwrap();
}

fn remove_item(database: &Connection) {
    print_prompt("Enter the ID of the item to remove: ");
    let result = get_item_selection();
    if result.is_err() {
        print_error(result.unwrap_err());
        return;
    }
    let choice = result.unwrap();
    let mut statement = database.prepare("DELETE FROM to_do_notes WHERE id = ?1").unwrap();
    let rows_deleted = statement.execute(params![choice]).unwrap();
    if rows_deleted == 0 {
        eprintln!("Chosen to do note does not exist.");
        return;
    }
}

fn get_item_selection() -> Result<i32, &'static str> {
    let result = read_line();
    if result.is_err() {
        return Err(result.unwrap_err());
    }
    let choice = result.unwrap();
    let parse_result = choice.parse::<i32>();
    if parse_result.is_err() {
        return Err("Invalid input for index.");
    }
    Ok(parse_result.unwrap())
}

fn read_line() -> Result<String, &'static str> {
    let mut input = String::new();
    let result = stdin().read_line(&mut input);
    if result.is_err() {
        return Err("Could not read input.");
    }
    Ok(input.trim_end().to_string())
}

fn print_prompt(prompt: &str) {
    let mut stdout = stdout();
    stdout.write_all(prompt.as_bytes()).unwrap();
    stdout.flush().unwrap();
}

fn print_error(error: &str) {
    let mut stderr = stderr();
    stderr.write_all(error.as_bytes()).unwrap();
    stderr.flush().unwrap();
}

fn main() {
    let user_data = app_root(AppDataType::UserData, &APP_INFO).unwrap();
    let database = Connection::open(user_data.join("data")).unwrap();
    database.execute(
        "CREATE TABLE IF NOT EXISTS to_do_notes ( \
            id INTEGER PRIMARY KEY AUTOINCREMENT, \
            title TEXT NOT NULL, \
            content TEXT NOT NULL, \
            created INTEGER, \
            done INTEGER \
        )",
        NO_PARAMS,
    ).unwrap();
    loop {
        print_prompt(
            "You have the following options:\n\
            c: Create new item.\n\
            l: List items.\n\
            d: Mark item done/undone.\n\
            r: Remove item from list.\n\
            q: Quit.\n\
            Please choose what to do: "
        );
        let result = read_line();
        if result.is_err() {
            print_error(result.unwrap_err());
            continue;
        }
        let choice = result.unwrap();
        match choice.as_ref() {
            "c" => create_item(&database),
            "l" => list_items(&database),
            "d" => mark_done(&database),
            "r" => remove_item(&database),
            "q" => break,
            _ => eprintln!("Invalid option! Try again.")
        }
    }
}
