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
#[derive(Clone)]
pub struct Hop {
    /// System indices of next hops.
    pub next: Vec<usize>,
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

/// Compute and rank all admissable at-most-single-via
/// alternative routes, returning up to *k* best. Based on a
/// metric from
///
/// > *Alternative Routes in Road Networks*  
/// > Ittai Abraham, Daniel Delling, Andrew V. Goldberg, Renato F. Werneck  
/// > Proc. Experimental Algorithms, 9th International Symposium (SEA 2010)  
/// > Naples, Italy May 2010  
///
/// Optimization constraints are:
///
/// * `max_routes`: Maximum number of routes to be returned
///    (including shortest).
/// * `sharing`: Maximum percentage of sharing of a route
///    with the shortest route.
/// * `local_opt`: Percentage of the shortest route length
///    over which the route must be locally optimal (all subroutes
///    of this length are shortest routes).
/// * `ub_stretch`: Percentage of "stretch" (extra jumps
///    beyond shortest route) allowed along any subroute of a
///    route.
///
/// The objective function is a heuristic based on the
/// settings of the optimization constraints.
///
/// If there is no route from `start` to `goal`, `None` will
/// be returned. Otherwise, the route list is guaranteed to
/// include at least the shortest route.
pub fn alt_routes(
    _map: &Map,
    _apsp: &APSPTable,
    _start: SystemId,
    _goal: SystemId,
    _max_routes: usize,
    _sharing: f64,
    _local_opt: f64,
    _ub_stretch: f64
    ) -> Vec<Vec<SystemId>>
{
    unimplemented!("compute alternate routes")
}

/// Reconstruct shortest routes from start to goal, if any,
/// using the APSP table.
pub fn shortest_routes_apsp(
    map: &Map,
    apsp: &APSPTable,
    start: SystemId,
    goal: SystemId,
    ) -> Option<Vec<Vec<SystemId>>>
{
    let systems = map.systems_ref();
    let mut start = map.by_system_id(start).system_index;
    let goal = map.by_system_id(goal).system_index;
    let full_hop = match apsp[[start, goal]] {
        Some(ref hop) => hop,
        None => return None,
    };
    let mut dist = full_hop.dist;
    let mut routes: Vec<Vec<SystemId>> = Vec::new();
    let mut route: Vec<SystemId> = Vec::new();
    route.push(systems[start].system_id);
    while start != goal {
        assert!(dist > 0);
        let next_neighbors = &apsp[[start, goal]]
            .as_ref()
            .expect("missing hop")
            .next;
        let n = next_neighbors.len();
        assert!(n > 0);
        if n > 1 {
            for neighbor in next_neighbors {
                let neighbor = &systems[*neighbor];
                let finishes =
                    shortest_routes_apsp(
                        map,
                        apsp,
                        neighbor.system_id,
                        systems[goal].system_id,
                        ).expect("could not extend route");
                for rest in finishes {
                    assert!(rest.len() == dist);
                    let mut full = route.clone();
                    full.extend(rest);
                    routes.push(full);
                }
            }
            return Some(routes);
        }
        let next = next_neighbors[0];
        route.push(systems[next].system_id);
        dist -= 1;
        start = next;
    }
    routes.push(route);
    Some(routes)
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
            if let &Some(ref hop) = &hops[[i, j]] {
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
    let mut hops: Array2<Option<Hop>> =
        Array2::from_elem((n, n), None);

    // Use iterated single-source shortest-path search to update
    // all hops.
    for start in systems {
        let j = start.system_index;
        let routes = bfs(map, start.system_id, None);
        for waypoint in routes.values() {
            let parent_info = match waypoint.parent {
                None => {
                    assert!(waypoint.cur == start.system_id);
                    continue;
                },
                Some(system_id) => {
                    map.by_system_id(system_id)
                },
            };
            let parent_index = parent_info.system_index;
            let cur_info = map.by_system_id(waypoint.cur);
            let i = cur_info.system_index;
            let mut old_hop = hops[[i, j]].take();
            match old_hop {
                Some(mut hop) => {
                    if hop.dist > waypoint.dist {
                        hop.dist = waypoint.dist;
                        hop.next = vec![parent_index];
                    } else if hop.dist == waypoint.dist {
                        hop.next.push(parent_index);
                        println!("multiple {:?}", hop.next);
                    }
                    hops[[i, j]] = Some(hop);
                },
                None => {
                    let new_hop = Some(Hop {
                        dist: waypoint.dist,
                        next: vec![parent_index],
                    });
                    hops[[i, j]] = new_hop;
                },
            }
        }
    }

    // Return the constructed table.
    hops
}
