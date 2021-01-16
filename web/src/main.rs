// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

// Plan B: EVE route planner with options
// Web client

use std::path::PathBuf;

#[macro_use]
extern crate rocket;
use rocket::request::{Form, State};
use rocket::response::NamedFile;
use rocket::Rocket;
use rocket::fairing::AdHoc;

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
async fn front_page(static_path: State<'_, StaticPath>) ->
    Result<NamedFile, rocket::response::Debug<std::io::Error>>
{
    let path = static_path.0.join("plan-b.html");
    Ok(NamedFile::open(path).await?)
}

// Display the Plan B favicon.
#[get("/favicon.ico")]
async fn favicon(static_path: State<'_, StaticPath>) ->
    Result<NamedFile, rocket::response::Debug<std::io::Error>>
{
    let path = static_path.0.join("plan-b-favicon.ico");
    Ok(NamedFile::open(path).await?)
}

// Process an EVE route request.
#[post("/", data = "<route_spec>")]
async fn search_route(
    route_spec: Form<RouteSpec>,
    map: State<'_, Map>,
) -> Option<String> {
    let from = map.by_name(&route_spec.from)?;
    let to = map.by_name(&route_spec.to)?;
    let route: Vec<String> =
        shortest_route(&map, from.system_id, to.system_id)?
            .iter()
            .map(|system_id| {
                map.by_system_id(*system_id).name.to_string()
            })
            .collect();
    Some(route.join("\n"))
}

// Plan B web service.
#[rocket::launch]
fn rocket() -> Rocket {
    async fn attach_path(rocket: Rocket) -> Result<Rocket, Rocket> {
        let static_path = ["web", "static"].iter().collect();
        Ok(rocket.manage(StaticPath(static_path)))
    }
                     
    Rocket::ignite()
        .attach(AdHoc::on_attach("Static Path", attach_path))
        .manage(Map::fetch().expect("could not load map"))
        .mount("/", routes![front_page, favicon, search_route])
}
