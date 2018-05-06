# Copyright © 2018 Po Huit
# [This program is licensed under the "MIT License"]
# Please see the file LICENSE in the source
# distribution of this software for license terms.

# Fetch EVE systems and stargates using ESI

from time import sleep
import http.client as client
import json
from sys import stdout, stderr
import threading

# Where to fetch the maps from.
esi_endpoint = "esi.tech.ccp.is"
# What version to fetch.
esi_version = "latest"
# Number of retries before giving up.
max_retries = 5
# How long to wait between retries (secs).
retry_timeout = 5.0
# How long to wait before reopening the connection (secs).
reopen_timeout = 5.0
# Delay inserted to max at given request rate (per sec).
request_rate = 20.0
# Number of simultaneous fetch threads to spawn.
nthreads = 20

# https://stackoverflow.com/a/312464
def chunks(l, n):
    "Yield n equal-sized chunks from l."
    nl = len(l)
    nchunk = nl // n
    for i in range(0, nl, nchunk):
        yield l[i:i + nchunk]

def log(*args):
    "Print to stdout and flush."
    print(*args)
    stdout.flush()

# Thread-local storage.
tls = threading.local()

def ccp_request(path):
    "Make an ESI request."
    url = "/" + esi_version + "/" + path + "/"
    for retries in range(max_retries):
        try:
            if retries == 1:
                sleep(reopen_timeout)
                tls.connection.close()
                tls.connection = client.HTTPSConnection(esi_endpoint)
            else:
                sleep(1.0/request_rate)
            tls.connection.request('GET', url)
            response = tls.connection.getresponse()
            if response.status == 200:
                try:
                    return json.load(response)
                except json.decoder.JSONDecodeError as e:
                    print("json error: ", e, file=stderr)
            else:
                print("bad response status: ", response.status, file=stderr)
        except client.HTTPException as e:
            print("http error: ", e.code, file=stderr)
        if retries < max_retries - 1:
            sleep(retry_timeout)
    print("fetch failed for", url, file=stderr)
    exit(1)

# Map of retrieved systems and stargates.
by_system_id = dict()
by_stargate_id = dict()

# A thread worker.
def worker(systems):
    "Fetch the given systems' information via ESI."
    global by_system_id, by_stargate_id
    tls.connection = client.HTTPSConnection(esi_endpoint)
    tls.by_system_id = dict()
    tls.by_stargate_id = dict()

    # Grab the systems.
    for system_id in systems:
        system = ccp_request('universe/systems/' + str(system_id))
        log(system['name'])
        tls.by_system_id[system_id] = system

    # Grab the stargates for each system.
    for system_id, system in tls.by_system_id.items():
        if 'stargates' not in system:
            continue
        stargates = system['stargates']
        for stargate_id in stargates:
            stargate = ccp_request('universe/stargates/' + str(stargate_id))
            log(system['name'], "->", stargate_id)
            tls.by_stargate_id[stargate_id] = stargate

    # Move system and stargate information to the global map.
    for system_id, system in tls.by_system_id.items():
        by_system_id[system_id] = system
    for stargate_id, stargate in tls.by_stargate_id.items():
        by_stargate_id[stargate_id] = stargate

# Open the master connection and get a list of systems.
tls.connection = client.HTTPSConnection(esi_endpoint)
systems = ccp_request('universe/systems')
nsystems = len(systems)
log(nsystems, "systems")

# Start and collect the threads.
threads = [threading.Thread(target=worker, args=(chunk,))
           for chunk in chunks(systems, nthreads)]
for t in threads:
    t.start()
for t in threads:
    t.join()

# Write the output JSON.
info = {'systems': by_system_id, 'stargates': by_stargate_id}
with open('eve-map.json', 'w') as dumpfile:
    json.dump(info, dumpfile)
