// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

// Plan B: EVE route planner with options
// Web client

// XXX The clippy allow here is due to the derivation from
// rocket-derive.
#![allow(clippy::unnecessary_lazy_evaluations)]

use std::path::PathBuf;

use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::*;

use plan_b::*;

// Need to wrap the EVE route spec for use in endpoints.
// XXX The word "route" is ambiguous in this code.
#[derive(FromForm)]
struct RouteSpec {
    from: String,
    to: String,
}

struct StaticPath(PathBuf);

// Display the Plan B front page.
#[get("/")]
async fn front_page(
    static_path: &State<StaticPath>,
) -> Result<NamedFile, rocket::response::Debug<std::io::Error>> {
    let path = static_path.0.join("plan-b.html");
    Ok(NamedFile::open(path).await?)
}

// Display the Plan B favicon.
#[get("/favicon.ico")]
async fn favicon(
    static_path: &State<StaticPath>,
) -> Result<NamedFile, rocket::response::Debug<std::io::Error>> {
    let path = static_path.0.join("plan-b-favicon.ico");
    Ok(NamedFile::open(path).await?)
}

// Process an EVE route request.
#[post("/", data = "<route_spec>")]
async fn search_route(route_spec: Form<RouteSpec>, map: &State<Map>) -> Option<String> {
    let from = map.by_name(&route_spec.from)?;
    let to = map.by_name(&route_spec.to)?;
    let route: Vec<String> = shortest_route(map, from.system_id, to.system_id)?
        .iter()
        .map(|system_id| map.by_system_id(*system_id).name.to_string())
        .collect();
    Some(route.join("\n"))
}

// Plan B web service.
#[launch]
fn rocket() -> Rocket<Build> {
    async fn attach_path(rocket: Rocket<Build>) -> Rocket<Build> {
        let static_path = ["web", "static"].iter().collect();
        rocket.manage(StaticPath(static_path))
    }

    rocket::build()
        .attach(AdHoc::on_ignite("Static Path", attach_path))
        .manage(Map::fetch().expect("could not load map"))
        .mount("/", routes![front_page, favicon, search_route])
}
