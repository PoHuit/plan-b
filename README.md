# Plan B
Copyright (c) 2018 Po Huit

Plan B is an [EVE Online](http://eveonline.com) route
planner. Plan B currently calculates a shortest route, but
is eventually intended to show reasonable alternative routes
also (hence the name).

This is very much a work-in-progress. See `vision.md` and
`reqs.md` in `docs/` for a roadmap and status.

## Usage

The core of this project is a Rust library "crate",
`plan_b`, containing most of the Plan B logic and
internals. There are two user interfaces provided to this
library: one command-line and one web.

### Run The Command-Line Client

Say `cargo run -p cmdline --release *start* *dest*` to find and
display a shortest route from the system named *start* to
the system named *dest*. The code will take a couple of
seconds to load the map, a millisecond or so to find and
display the route, and then will print all the hops, one per
line, on stdout.

Say `cargo run -p cmdline --release -- --diameter` to
calculate the
[diameter](http://schildwall.phbv3.de/topology.html)
of New Eden and show the endpoints of the three longest
shortest routes. The code will take about 10 seconds to
compute the answer.

## Build

This project is written mostly in Rust, with a little Python
3 for convenience. It has only been used/tested on Linux.
The build process is reasonably simple.

### Get The Map Data (Optional)

I have included `eve-map.json.gz` in the repository
top-level containing EVE System and Stargate data in
compressed JSON. Copy this file to `/usr/local/share` on
your box and you should be set. Since EVE Systems and
Stargates normally never change, this should be the standard
way to set things up.

If you want to regenerate this data, go to the `fetch-map`
directory and run `python3 fetch-map.py`. This should run
for about 30 minutes with a decent Internet connection, and
will create `eve-map.json` by fetching the necessary data
from [CCP](https://www.ccpgames.com/)'s Tranquility server
using
[ESI](http://eveonline-third-party-documentation.readthedocs.io/en/latest/esi/).
Compress this file with `gzip` and you're ready to proceed
as above.

### Build The Rust Code

1. Get the latest stable Rust installed on your system.

2. Run `cargo build --release` from the top-level directory.

The build will take a minute or so, and then the code should
be ready to go.

## License

This program is licensed under the "MIT License".  Please
see the file LICENSE in the source distribution of this
software for license terms.

## Acknowlegements

Thanks to Sparky Doosan for the name. Thanks to my EVE
Online software development class for being part of the
project.
