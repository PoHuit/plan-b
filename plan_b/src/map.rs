// Copyright © 2018 Po Huit
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.


// Map data management for Plan B

extern crate libflate;
use self::libflate::gzip;

extern crate serde_json;
use self::serde_json::Value;

use std::error::Error;
use std::collections::HashMap;
use std::fs::File;
use std::fmt;
use std::slice;

#[derive(Debug)]
struct MapDataError(&'static str);

impl Error for MapDataError {
    fn description(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for MapDataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "map data error: {}", self.description())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemId(usize);

#[derive(Debug)]
pub struct SystemInfo {
    pub system_id: SystemId,
    pub name: String,
    pub stargates: Vec<SystemId>,
}

#[derive(Debug)]
pub struct Map {
    systems: Vec<SystemInfo>,
    by_system_id: HashMap<SystemId, usize>,
    by_name: HashMap<String, usize>,
}

impl Map {
    pub fn fetch() -> Result<Map, Box<Error>> {
        let map_file = File::open("eve-map.json.gz")?;
        let gunzip = gzip::Decoder::new(map_file)?;
        let map_data: Value = serde_json::from_reader(gunzip)?;
        let json_systems = map_data["systems"]
            .as_object()
            .ok_or_else(|| MapDataError("no systems"))?;
        let json_stargates = map_data["stargates"]
            .as_object()
            .ok_or_else(|| MapDataError("no stargates"))?;
        let mut by_system_id = HashMap::new();
        let mut by_name = HashMap::new();
        let mut systems = Vec::with_capacity(json_systems.len());
        let mut system_index = 0;
        for (system_id_str, system) in json_systems {
            let system_id = SystemId(system_id_str.parse().unwrap());

            let name = system["name"]
                .as_str()
                .ok_or_else(|| MapDataError("no system name"))?
                .to_string();

            let mut stargates = Vec::new();
            let stargate_ids =
                match system.get("stargates") {
                    None => continue,
                    Some(array) => array
                        .as_array()
                        .ok_or_else(|| MapDataError("bad system stargates"))?,
                };
            for stargate_id in stargate_ids {
                let stargate_id_string = stargate_id.to_string();
                let dest_id = json_stargates[&stargate_id_string]
                    .as_object()
                    .ok_or_else(|| MapDataError("bad stargate id"))?
                    .get("destination")
                    .ok_or_else(|| MapDataError("no stargate destination"))?
                    .get("system_id")
                    .ok_or_else(|| MapDataError("no stargate system id"))?
                    .as_u64()
                    .ok_or_else(|| MapDataError("bad stargate system_id"))?;
                stargates.push(SystemId(dest_id as usize));
            }

            let system_info = SystemInfo {
                system_id,
                name: name.clone(),
                stargates
            };
            systems.push(system_info);
            by_system_id.insert(system_id, system_index);
            by_name.insert(name, system_index);

            system_index += 1;
        }
        Ok(Map{systems, by_system_id, by_name})
    }

    pub fn by_name<'a>(&'a self, name: &'a str) -> Option<&'a SystemInfo> {
        self
            .by_name.get(name)
            .map(|i| &self.systems[*i])
    }

    pub fn by_system_id<'a>(&'a self, id: SystemId) -> &'a SystemInfo {
        let i = self
            .by_system_id.get(&id)
            .expect("by_system_id: invalid SystemId");
        &self.systems[*i]
    }

    pub fn systems<'a>(&'a self) -> slice::Iter<'a, SystemInfo> {
        self.systems.iter()
    }
}
