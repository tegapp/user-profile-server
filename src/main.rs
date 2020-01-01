#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

pub mod schema;
pub mod user;
pub mod machine;

use self::user::{ User, NewUser };
use std::io::{stdin, Read};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn create_user<'a>(conn: &PgConnection, auth0_id: &'a str) -> User {
    use schema::users;

    let new_user = NewUser {
        auth0_id,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(conn)
        .expect("Error saving new post")
}

fn main() {
}
//     let connection = establish_connection();

//     println!("What would you like your auth0 id to be?");
//     let mut title = String::new();
//     stdin().read_line(&mut title).unwrap();
//     let title = &title[..(title.len() - 1)]; // Drop the newline character

//     let post = create_user(&connection, title);
//     println!("\nSaved draft {} with id {}", title, post.id);
// }

// #[cfg(not(windows))]
// const EOF: &'static str = "CTRL+D";

// #[cfg(windows)]
// const EOF: &'static str = "CTRL+Z";