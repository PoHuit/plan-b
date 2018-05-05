// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

extern crate ndarray;

use self::ndarray::Array2;
    
use std::collections::VecDeque;
use std::collections::HashMap;

use map::*;

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
    dist: usize,
    cur: SystemId,
    parent: Option<SystemId>,
}

impl Waypoint {
    fn new(dist: usize, cur: SystemId, parent: Option<SystemId>)
           -> Waypoint
    {
        Waypoint{dist, cur, parent}
    }
}

fn dijkstra(map: &Map, start: SystemId, goal: Option<SystemId>)
            -> HashMap<SystemId, Waypoint>
{
    let mut q = VecDeque::with_capacity(map.systems_ref().len());
    let mut closed = HashMap::new();
    q.push_back(Waypoint::new(0, start, None));
    loop {
        let waypoint = match q.pop_front() {
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
            q.push_back(child_waypoint);
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

    for start in systems {
        let j = start.system_index;
        let routes = dijkstra(map, start.system_id, None);
        for waypoint in routes.values() {
            if waypoint.parent.is_none() {
                continue;
            }
            let cur = map.by_system_id(waypoint.cur);
            let i = cur.system_index;
            match hops[[i, j]] {
                Some(hop) if hop.dist <= waypoint.dist => continue,
                _ => hops[[i, j]] = Some( Hop{
                    system_id: waypoint.parent.expect("apsp: walked off map"),
                    dist: waypoint.dist,
                }),
            }
        }
    }

    hops
}
