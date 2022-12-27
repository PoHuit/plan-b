// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

// Plan B: EVE route planner with options
// Web client

use std::sync::Arc;

use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::*,
};
use serde::Deserialize;

use plan_b::*;

// Display the Plan B front page.
async fn front_page(_: State<Arc<Map>>) -> Html<String> {
    let html = std::fs::read_to_string("static/plan-b.html").unwrap();
    Html(html)
}

// Display the Plan B favicon.
async fn favicon(_: State<Arc<Map>>) -> Result<Response, (StatusCode, String)> {
    let favicon = std::fs::read("static/plan-b-favicon.ico").map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("internal error: could not find favicon: {e}"),
        )
    })?;
    let response = ([("CONTENT_TYPE", "image/vnd.microsoft.icon")], favicon);
    Ok(response.into_response())
}

// Need to wrap the EVE route spec for use in endpoints.
// XXX The word "route" is ambiguous in this code.
#[derive(Deserialize)]
struct RouteSpec {
    from: String,
    to: String,
}

// Process an EVE route request.
// https://github.com/joelparkerhenderson/demo-rust-axum/
// examples/html-form-get-and-post
async fn search_route(
    State(map): State<Arc<Map>>,
    form: Form<RouteSpec>,
) -> Result<String, (StatusCode, String)> {
    let from = map.by_name(&form.0.from).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            format!("from: system {} not found", &form.0.from),
        )
    })?;
    let to = map.by_name(&form.0.to).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            format!("to: system {} not found", &form.0.to),
        )
    })?;
    let route: Vec<&str> = shortest_route(&map, from.system_id, to.system_id)
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                format!("no route found from {} to {}", &from.name, &to.name,),
            )
        })?
        .iter()
        .map(|&system_id| map.by_system_id(system_id).name.as_ref())
        .collect();
    Ok(route.join("\n"))
}

// Plan B web service.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let map = Map::fetch().expect("internal error: could not find map data: {e}");
    let map = Arc::new(map);

    let app = Router::new()
        .route("/", get(front_page))
        .route("/favicon.ico", get(favicon))
        .route("/", post(search_route))
        .with_state(map);
    axum::Server::bind(&"0.0.0.0:9146".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
