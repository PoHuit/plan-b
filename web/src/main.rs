// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

// Plan B: EVE route planner with options
// Web client

#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
use rocket::request::{Form, State};
use rocket::response::NamedFile;

extern crate plan_b;
use plan_b::*;

// Need to wrap the EVE route spec for use in endpoints.
// XXX The word "route" is ambiguous in this code.
#[derive(FromForm)]
struct RouteSpec {
    from: String,
    to: String,
}

// Display the Plan B front page.
#[get("/")]
fn front_page() -> std::io::Result<NamedFile> {
    NamedFile::open("static/plan-b.html")
}

// Process an EVE route request.
#[post("/", data = "<route_spec>")]
fn search_route(route_spec: Form<RouteSpec>, map: State<Map>) -> Option<String> {
    let route_spec = route_spec.get();
    let from = map.by_name(&route_spec.from)?;
    let to = map.by_name(&route_spec.to)?;
    let route: Vec<String> =
        shortest_route(&map, from.system_id, to.system_id)?
            .iter()
            .map(|system_id| map.by_system_id(*system_id).name.to_string())
            .collect();
    Some(route.join("\n"))
}

// Plan B web service.
fn main() {
    rocket::ignite()
        .manage(Map::fetch().expect("could not load map"))
        .mount("/", routes![front_page, search_route])
        .launch();
}
