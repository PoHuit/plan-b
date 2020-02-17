// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

// Plan B: EVE route planner with options
// Command-line demo client

use structopt::StructOpt;

use plan_b::*;

// Command-line arguments

#[derive(StructOpt, Debug)]
#[structopt(name = "plan-b")]
struct Opt {
    #[structopt(short = "d", long = "diameter")]
    diameter: bool,
    #[structopt(short = "a", long = "all")]
    all: bool,
    #[structopt(name = "START")]
    start: String,
    #[structopt(name = "GOAL")]
    goal: String,
}

// Look up the given system name in the map, and panic if
// not found. This should be cleaned up.
fn find_system(map: &Map, name: &str) -> SystemId {
    map.by_name(name)
        .unwrap_or_else(|| panic!("could not find {} in map", name))
        .system_id
}

// Find a shortest route by name, or panic if none exists.
fn find_route(map: &Map, start: &str, goal: &str) -> Vec<SystemId> {
    let start_id = find_system(&map, start);
    let goal_id = find_system(&map, goal);
    shortest_route(&map, start_id, goal_id).unwrap_or_else(|| {
        panic!("no route found from {} to {}", start, goal)
    })
}

// Find all shortest routes by name, or panic if none exists.
fn find_all_routes(
    map: &Map,
    start: &str,
    goal: &str,
) -> Vec<Vec<SystemId>> {
    let start_id = find_system(&map, start);
    let goal_id = find_system(&map, goal);
    let apsp = apsp(&map);
    shortest_routes_apsp(&map, &apsp, start_id, goal_id).unwrap_or_else(
        || panic!("no route found from {} to {}", start, goal),
    )
}

#[test]
// Check for correct computation of a long route.
fn short_route_north_south() {
    let map = Map::fetch().expect("could not open map");
    let route = find_route(&map, "B-GC1T", "2UK4-N");
    assert_eq!(80, route.len());
}

// Display a given route, one system per line.
fn show_route(map: &Map, route: &[SystemId]) {
    for system_id in route {
        let system = map.by_system_id(*system_id);
        println!("{}", system.name);
    }
}

// Command-line Plan B. */
fn main() {
    // Set up the map.
    let map = Map::fetch().expect("could not open map");

    // Get and process the arguments.
    let opt = Opt::from_args();

    // Just do diameter.
    if opt.diameter {
        // Run the diameter calculation and display the result.
        let diameter_info = diameter(&map);
        println!("diameter {}", diameter_info.diameter);
        for (start, end) in diameter_info.longest {
            let start = &map.by_system_id(start).name;
            let end = &map.by_system_id(end).name;
            println!("{} → {}", start, end);
        }
        return;
    }

    // Show all routes.
    if opt.all {
        let mut routes = find_all_routes(&map, &opt.start, &opt.goal);
        let last = routes.pop().unwrap();
        for route in routes {
            show_route(&map, &route);
            println!();
        }
        show_route(&map, &last);
        return;
    }

    // Get the destination, find the route and display it.
    let route = find_route(&map, &opt.start, &opt.goal);
    show_route(&map, &route);
}
