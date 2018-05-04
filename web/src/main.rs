// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

// Plan B: EVE route planner with options
// Web client

#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
use rocket::request::Form;
use rocket::response::NamedFile;

extern crate plan_b;
use plan_b::*;

#[derive(FromForm)]
struct RouteSpec {
    from: String,
    to: String,
}

#[get("/")]
fn front_page() -> std::io::Result<NamedFile> {
    NamedFile::open("static/plan-b.html")
}

#[post("/route", data = "<route_spec>")]
fn search_route(route_spec: Form<RouteSpec>) -> String {
    let route_spec = route_spec.get();
    format!("{} -> {}", route_spec.from, route_spec.to)
}

fn main() {
    rocket::ignite().mount("/", routes![front_page, search_route]).launch();
}
