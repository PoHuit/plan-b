// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

// Plan B: EVE route planner with options
// Command-line demo client

extern crate plan_b;
use plan_b::map::*;
use plan_b::search::*;

fn find_system(map: &Map, name: &str) -> SystemId {
    map.by_name(name)
        .expect(&format!("could not find {} in map", name))
        .system_id
}

fn find_route(map: &Map, start: &str, goal: &str) -> Vec<SystemId> {
    let start_id = find_system(&map, start);
    let goal_id = find_system(&map, goal);
    shortest_route(&map, start_id, goal_id)
        .expect(&format!("no route found from {} to {}", start, goal))
}

#[test]
fn shortest_route_north_south() {
    let map = Map::fetch().expect("could not open map");
    let route = find_route(&map, "B-GC1T", "2UK4-N");
    assert_eq!(80, route.len());
}

fn main() {
    let map = Map::fetch().expect("could not open map");
    let mut args = std::env::args();
    let start = (&mut args).skip(1).next().expect("no source");
    if start == "--diameter" {
        let diameter_info = diameter(&map);
        println!("diameter {}", diameter_info.diameter);
        for (start, end) in diameter_info.longest {
            let start = &map.by_system_id(start).name;
            let end = &map.by_system_id(end).name;
            println!("{} → {}", start, end);
        }
        return;
    }
    let goal = (&mut args).next().expect("no destination");
    let route = find_route(&map, &start, &goal);
    for system_id in route {
        let system = map.by_system_id(system_id);
        println!("{}", system.name);
    }
}
