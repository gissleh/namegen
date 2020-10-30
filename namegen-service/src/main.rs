#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde;
extern crate serde_json;
extern crate namegen;

mod store;
mod api;

use crate::store::Store;

fn main() {
    rocket::ignite()
        .manage(Store::new())
        .mount("/", routes![
            api::name::list_names,
            api::name::get_name,
            api::name::create_name,
            api::name::generate_name,
            api::name::delete_name,
            api::format::list_formats,
            api::format::add_format,
        ])
        .launch();
}