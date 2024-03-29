// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Map data management for Plan B.

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::slice;

use libflate::gzip;

/// A `SystemId` as defined by CCP.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemId(usize);

/// Map info on a given system.
#[derive(Debug)]
pub struct SystemInfo {
    /// `SystemId` of this system.
    pub system_id: SystemId,
    /// Name of this system.
    pub name: String,
    /// `SystemId`s of systems connected to this one
    /// via outgoing stargates.
    pub stargates: Vec<SystemId>,
    /// Index into the `Map`'s internal system list.
    pub system_index: usize,
}

/// The map, containing info needed for routing.
#[derive(Debug)]
pub struct Map {
    systems: Vec<SystemInfo>,
    by_system_id: HashMap<SystemId, usize>,
    by_name: HashMap<String, usize>,
}

// JSON representations of map data as Rust structs.
mod json_repr {
    use std::collections::HashMap;

    use serde::{de, Deserialize, Deserializer};
    use serde_json::{self, Value};
    use std::str::FromStr;

    fn de_from_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Value::deserialize(deserializer)?;
        let e = "an f64 literal or string";
        match v {
            Value::Number(v) => Ok(v.as_f64().unwrap()),
            Value::String(s) => f64::from_str(&s).map_err(de::Error::custom),
            Value::Null => {
                let ue = de::Unexpected::Other("null");
                Err(de::Error::invalid_value(ue, &e))
            }
            Value::Bool(b) => {
                let ue = de::Unexpected::Bool(b);
                Err(de::Error::invalid_value(ue, &e))
            }
            Value::Array(_) => {
                let ue = de::Unexpected::Seq;
                Err(de::Error::invalid_value(ue, &e))
            }
            Value::Object(_) => {
                let ue = de::Unexpected::Map;
                Err(de::Error::invalid_value(ue, &e))
            }
        }
    }

    #[derive(Deserialize)]
    pub struct Destination {
        pub stargate_id: usize,
        pub system_id: usize,
    }

    #[derive(Deserialize)]
    pub struct Point {
        #[serde(deserialize_with = "de_from_f64")]
        pub x: f64,
        #[serde(deserialize_with = "de_from_f64")]
        pub y: f64,
        #[serde(deserialize_with = "de_from_f64")]
        pub z: f64,
    }

    #[derive(Deserialize)]
    pub struct Stargate {
        pub destination: Destination,
        pub name: String,
        pub position: Point,
        pub stargate_id: usize,
        pub system_id: usize,
        pub type_id: usize,
    }

    #[derive(Deserialize)]
    pub struct Planet {
        pub asteroid_belts: Option<Vec<usize>>,
        pub moons: Option<Vec<usize>>,
        pub planet_id: usize,
    }

    #[derive(Deserialize)]
    pub struct System {
        pub constellation_id: usize,
        pub name: String,
        pub planets: Option<Vec<Planet>>,
        pub position: Point,
        pub security_class: Option<String>,
        pub security_status: f64,
        pub star_id: Option<usize>,
        pub stargates: Option<Vec<usize>>,
        pub stations: Option<Vec<usize>>,
        pub system_id: usize,
    }

    #[derive(Deserialize)]
    pub struct Map {
        pub stargates: HashMap<usize, Stargate>,
        pub systems: HashMap<usize, System>,
    }
}

fn find_map_file() -> Result<File, Box<dyn Error>> {
    let mut f = None;
    for fname in [
        "../eve-map.json.gz",
        "./static/eve-map.json.gz",
        "./eve-map.json.gz",
        "/usr/local/share/eve-map.json.gz",
    ]
    .iter()
    {
        f = Some(File::open(fname));
        if let Some(Ok(f)) = f {
            return Ok(f);
        }
    }
    f.unwrap().map_err(|e| Box::new(e) as Box<dyn Error>)
}

impl Map {
    /// Retrieve and parse the map data.
    pub fn fetch() -> Result<Map, Box<dyn Error>> {
        // Load up the JSON map data.
        let map_file = find_map_file()?;
        let gunzip = gzip::Decoder::new(map_file)?;
        let map: json_repr::Map = serde_json::from_reader(gunzip)?;

        // Set up the state and process the data.
        let mut by_system_id = HashMap::new();
        let mut by_name = HashMap::new();
        let mut systems = Vec::with_capacity(map.systems.len());
        let mut system_index = 0;
        for (system_id, system) in &map.systems {
            // Parse the current system id.
            let system_id = SystemId(*system_id);

            // Process the system stargates.
            let stargates = match system.stargates {
                None => continue,
                Some(ref stargate_ids) => {
                    stargate_ids
                        .iter()
                        .map(|s| SystemId(map.stargates[s].destination.system_id))
                        .collect()
                }
            };

            // Save the system info and update the hashmaps.
            let system_info = SystemInfo {
                system_id,
                name: system.name.clone(),
                stargates,
                system_index,
            };
            systems.push(system_info);
            by_system_id.insert(system_id, system_index);
            by_name.insert(system.name.clone(), system_index);

            // Increase the system index for the next round.
            system_index += 1;
        }
        // Return the now-completed map.
        Ok(Map {
            systems,
            by_system_id,
            by_name,
        })
    }

    /// Return some reference to the system info for the system
    /// with the given name, if found.
    pub fn by_name<'a>(&'a self, name: &'a str) -> Option<&'a SystemInfo> {
        self.by_name.get(name).map(|i| &self.systems[*i])
    }

    /// Return some reference to the system info for the system
    /// with the given system id.
    pub fn by_system_id(&self, id: SystemId) -> &SystemInfo {
        let i = self
            .by_system_id
            .get(&id)
            .expect("by_system_id: invalid SystemId");
        &self.systems[*i]
    }

    /// Return an iterator over the system info of all
    /// systems in the map.
    pub fn systems(&self) -> slice::Iter<'_, SystemInfo> {
        self.systems.iter()
    }

    /// Return a slice of all systems in the map.
    pub fn systems_ref(&self) -> &[SystemInfo] {
        &self.systems
    }
}
