// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

extern crate min_max_heap;
extern crate ndarray;

use self::min_max_heap::MinMaxHeap;
use self::ndarray::Array2;
    
use map::*;

use std::collections::HashMap;

pub struct DiameterInfo {
    pub diameter: usize,
    pub longest: Vec<(SystemId, SystemId)>,
}

#[derive(Clone, Copy)]
pub struct Hop {
    pub system_id: SystemId,
    pub dist: usize,
}

pub type APSPTable = Array2<Option<Hop>>;

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

pub fn diameter(map: &Map) -> DiameterInfo {
    let systems = map.systems_ref();
    let hops = apsp(map);
    let n = systems.len();
    let mut diameter = 0;
    let mut longest = Vec::new();
    for i in 0..n {
        for j in i+1..n {
            if let Some(hop) = hops[[i, j]] {
                let dist = hop.dist;
                if dist > diameter {
                    diameter = dist;
                    longest.clear();
                }
                if dist == diameter {
                    let iid = systems[i].system_id;
                    let jid = systems[j].system_id;
                    longest.push((iid, jid));
                }
            }
        }
    }
    DiameterInfo{diameter, longest}
}

pub fn apsp(map: &Map) -> APSPTable {
    let systems = map.systems_ref();
    let n = systems.len();
    let mut hops = Array2::from_elem((n, n), None);

    for i in 0..n {
        for next_hop in systems[i].stargates.iter() {
            let next_hop = map.by_system_id(*next_hop);
            let j = next_hop.system_index;
            hops[[i, j]] = Some( Hop{
                system_id: next_hop.system_id,
                dist: 1,
            });
        }
    }

    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                if let (Some(u), Some(w)) =
                    (hops[[i, k]], hops[[k, j]])
                {
                    let d_uw = u.dist + w.dist;
                    match hops[[i, j]] {
                        Some(v) if v.dist <= d_uw => continue,
                        _ => hops[[i, j]] = Some( Hop{
                            system_id: u.system_id,
                            dist: d_uw,
                        }),
                    }
                }
            }
        }
    }

    hops
}
