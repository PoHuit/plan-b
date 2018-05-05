// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

extern crate min_max_heap;
use self::min_max_heap::MinMaxHeap;

use map::*;

use std::collections::HashMap;

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
struct Waypoint {
    dist: u32,
    cur: SystemId,
    parent: Option<SystemId>,
}

impl Waypoint {
    fn new(dist: u32, cur: SystemId, parent: Option<SystemId>)
           -> Waypoint
    {
        Waypoint{dist, cur, parent}
    }
}

fn dijkstra(map: &Map, start: SystemId, goal: Option<SystemId>)
            -> HashMap<SystemId, Waypoint>
{
    let mut q = MinMaxHeap::new();
    let mut closed = HashMap::new();
    q.push(Waypoint::new(0, start, None));
    loop {
        let waypoint = match q.pop_min() {
            Some(waypoint) => waypoint,
            None => return closed,
        };
        if closed.contains_key(&waypoint.cur) {
            continue;
        }
        closed.insert(waypoint.cur, waypoint.clone());
        if goal == Some(waypoint.cur) {
            return closed;
        }
        let map_info = map.by_system_id(waypoint.cur);
        for child in map_info.stargates.iter() {
            let child_waypoint = Waypoint::new(
                waypoint.dist + 1,
                *child,
                Some(waypoint.cur),
            );
            q.push(child_waypoint);
        }
    }
}
    

pub fn shortest_route(map: &Map, start: SystemId, goal: SystemId)
                      -> Option<Vec<SystemId>>
{
    let waypoints = dijkstra(map, start, Some(goal));
    let cur = waypoints.get(&goal)?;
    let mut route = Vec::with_capacity(cur.dist as usize);
    let mut next_stop = cur.parent;
    route.push(cur.cur);
    while let Some(system_id) = next_stop {
        route.push(system_id);
        let cur = waypoints[&system_id].clone();
        next_stop = cur.parent;
    }
    route.reverse();
    Some(route)
}

pub fn diameter(map: &Map) {
    let systems: Vec<&SystemInfo> = map
        .systems()
        .collect();
    let system_ids: Vec<SystemId> = systems
        .iter()
        .map(|s| s.system_id)
        .collect();
    let mut diameter = 0;
    let mut routes_searched = 0;
    let mut endpoints = Vec::new();
    println!("searching {} systems", systems.len());
    for i in 0..system_ids.len() {
        let start = system_ids[i];
        let waypoints = dijkstra(map, start, None);
        for waypoint in waypoints.values()  {
            if waypoint.dist > diameter {
                diameter = waypoint.dist;
                endpoints.clear();
            }
            if waypoint.dist == diameter {
                endpoints.push((start, waypoint.cur));
            }
            routes_searched += 1;
        }
    }
    println!("diameter {} ({} routes searched)",
             diameter, routes_searched);
    for (start, end) in endpoints {
        if start < end {
            println!("{} → {}",
                     map.by_system_id(start).name,
                     map.by_system_id(end).name);
        }
    }
}
