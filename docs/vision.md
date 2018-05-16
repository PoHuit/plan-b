# "Plan B": An EVE Route Planner With Options
Copyright (c) 2018 Po Huit

*Plan B (thanks to Cousin Rob for the name) is an EVE route
planner that offers reasonable alternative routes.* Plan B
was inspired by the alternate routes facility in Google
Maps. Sometimes the "best" route isn't quite the
fastest/shortest one: this tool can give you options.

Plan B will allow selecting start, end and waypoints from
any named system, your current location taken from EVE,
location of personal assets, or commonly-used systems such
as trade hubs. Given the route specification, Plan B will
compute some number of differentiated routes optimized for
travel time. It can enter a sparsified set of waypoints for
the route into EVE, or can present successive jumps as you
travel through space.

## Partial Roadmap

* Version 0.1, 1 May 2018: Command-line tool. Accepts
  starting and ending system by name. Computes shortest
  route and displays it in a reasonable ASCII format.

* Version 0.2, 8 May 2018: Presents as a browser app.

* Version 0.3, 15 May 2018: Computes *k*-best routes
  and displays them in a reasonable ASCII format.

* Version 0.4, 22 May 2018: Acquires current location via
  ESI. Can post selected route by setting waypoints via ESI.
  Shows next jump during travel.

* Version 0.3, 29 May 2018: Provides a visual route map with
  a route selection interface. Heuristic prioritizes
  differentiated routes.
