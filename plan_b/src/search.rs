// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

extern crate min_max_heap;
use self::min_max_heap::MinMaxHeap;

use map::*;

use std::collections::HashMap;

pub fn shortest_route(map: &Map,
                      start: SystemId,
                      goal: SystemId,
                      ) -> Option<Vec<SystemId>> {
    let mut q = MinMaxHeap::new();
    let mut closed: HashMap<SystemId, (usize, Option<SystemId>)>
        = HashMap::new();
    q.push((0, start, None));
    loop {
        let (dist, cur, parent) = q.pop_min()?;
        if closed.contains_key(&cur) {
            continue;
        }
        closed.insert(cur, (dist, parent));
        if cur == goal {
            let mut route = Vec::with_capacity(dist);
            route.push(cur);
            let mut next_stop = parent;
            while let Some(system_id) = next_stop {
                route.push(system_id);
                let (_, parent) = closed[&system_id];
                next_stop = parent;
            }
            route.reverse();
            return Some(route);
        }
        let map_info = map.by_system_id(cur);
        for child in map_info.stargates.iter() {
            q.push((dist + 1, *child, Some(cur)));
        }
    }
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
    let mut max_start = None;
    let mut max_goal = None;
    println!("searching {} systems", systems.len());
    for i in 0..system_ids.len() {
        println!("{} -> ({} routes searched)",
                systems[i].name,
                routes_searched);
        let start = system_ids[i];
        for j in i+1..system_ids.len() {
            let goal = system_ids[j];
            if let Some(route) = shortest_route(map, start, goal) {
                if route.len() > diameter {
                    diameter = route.len();
                    max_start = Some(i);
                    max_goal = Some(j);
                }
            }
            routes_searched += 1;
        }
    }
    println!("diameter {} for {} -> {} ({} routes searched)",
             diameter,
             systems[max_start.unwrap()].name,
             systems[max_goal.unwrap()].name,
             routes_searched);
}
