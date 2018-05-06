// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Search functionality for Plan B.

extern crate ndarray;

use self::ndarray::Array2;
    
use std::collections::VecDeque;
use std::collections::HashMap;

use map::*;

/// Results from a `diameter()` calculation.
pub struct DiameterInfo {
    /// Diameter of EVE.
    pub diameter: usize,
    /// List of endpoints with shortest route of
    /// length equal to diameter.
    pub longest: Vec<(SystemId, SystemId)>,
}

/// An entry in the all-pairs shortest-path table.
#[derive(Clone, Copy)]
pub struct Hop {
    /// System id of next hop.
    pub system_id: SystemId,
    /// Distance from start to here.
    pub dist: usize,
}

/// Table of all-pairs shortest paths.
pub type APSPTable = Array2<Option<Hop>>;

/// An intermediate step in the BFS shortest path search.
#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
struct Waypoint {
    /// Distance from start.
    dist: usize,
    /// System id of current system.
    cur: SystemId,
    /// Next hop back toward parent, if not at start.
    parent: Option<SystemId>,
}

impl Waypoint {
    /// Create a new waypoint; mild syntactic sugar.
    fn new(dist: usize, cur: SystemId, parent: Option<SystemId>)
           -> Waypoint
    {
        Waypoint{dist, cur, parent}
    }
}

// Single-source shortest path via Breadth-First Search.
// Returns a waypoint map for further processing.
fn bfs(map: &Map, start: SystemId, goal: Option<SystemId>)
            -> HashMap<SystemId, Waypoint>
{
    // Set up data structures and run the search.
    let mut q = VecDeque::with_capacity(map.systems_ref().len());
    let mut closed = HashMap::new();
    q.push_back(Waypoint::new(0, start, None));
    loop {
        // Examine best waypoint.
        let waypoint = match q.pop_front() {
            Some(waypoint) => waypoint,
            None => return closed,
        };
        if closed.contains_key(&waypoint.cur) {
            continue;
        }
        closed.insert(waypoint.cur, waypoint.clone());

        // If we have found the goal, we are done.
        if goal == Some(waypoint.cur) {
            return closed;
        }

        // Open the children of the current system.
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
    

/// Return a shortest route if one exists.
pub fn shortest_route(map: &Map, start: SystemId, goal: SystemId)
                      -> Option<Vec<SystemId>>
{
    // Find single-source shortest paths from start up to goal.
    let waypoints = bfs(map, start, Some(goal));

    // Set up state and walk route.
    let cur = waypoints.get(&goal)?;
    let mut route = Vec::with_capacity(cur.dist as usize);
    let mut next_stop = cur.parent;
    route.push(cur.cur);
    while let Some(system_id) = next_stop {
        route.push(system_id);
        let cur = waypoints[&system_id].clone();
        next_stop = cur.parent;
    }

    // Route was walked in reverse order. Reverse and return
    // it.
    route.reverse();
    Some(route)
}

/// Compute the diameter of New Eden, with other interesting
/// info.
pub fn diameter(map: &Map) -> DiameterInfo {
    // Collect needed info.
    let systems = map.systems_ref();
    let hops = apsp(map);
    let n = systems.len();

    // Reconstruct max diameter and incrementally update
    // max endpoints.
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

    // Return the accumulated info.
    DiameterInfo{diameter, longest}
}

/// Compute an all-pairs shortest-path route table.
pub fn apsp(map: &Map) -> APSPTable {
    // Set up necessary info.
    let systems = map.systems_ref();
    let n = systems.len();
    let mut hops = Array2::from_elem((n, n), None);

    // Set up initial hops. XXX This is probably not needed
    // anymore.
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

    // Use iterated single-source shortest-path search to update
    // all hops.
    for start in systems {
        let j = start.system_index;
        let routes = bfs(map, start.system_id, None);
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

    // Return the constructed table.
    hops
}
