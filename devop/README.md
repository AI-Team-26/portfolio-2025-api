# Devop

Ports:
- 8300: website
- 8301: api
- 8302: database

## Docker containers for API and Database

From the "devop" folder:  
`docker compose -f compose.all.yaml up`  or  
`docker compose -f compose.all.yaml up --build`  
**note:** if you change the _.env_ or the _configuration_ files, use **--buil** version.  

## Database

The database is a Postgres docker container running on a Linux VPS, not part of this project.    
We use [SQLx](https://docs.rs/sqlx/latest/sqlx) to manage database interaction.  
We use SQLx macros that check the SQL over the database, see the [doc about SQLx](../src/repositories/SQLx.md).  

### Local database

Using _docker-compose_  we can run both th API and the database.  
Run ``docker compose -f compose.database.yaml up`` from this folder to run/start the database only,   
it will use the _.env_ file (git-ignored) to get the environment variables for secrets.  

### Connection string

The database connection string is stored in the Configuration.  
The Configuration is filled with a _configuration.json_ file.  
For local development we have a git-ignored file in the solution,  
for remote environment a **CONFIGURATION_FILE** environment variable should indicates where to read that file.

### Database pool tuning

Pool sizing is explicit via the `database_pool` configuration block instead of sqlx defaults:

| Key | Default suggestion | Rationale |
|-----|--------------------|-----------|
| `max_connections` | 20 | Postgres is process-per-connection: each open connection costs ~10MB+ RAM and a backend process. 20 covers realistic concurrent request load without pressuring the DB host; raise only if pool saturation is observed under real traffic. |
| `min_connections` | 4 | Keeps a small warm set so cold starts don't pay TCP+auth latency per request; low enough to free memory when idle. |
| `acquire_timeout_secs` | 5 | Bounds how long a handler waits for a free connection. A saturated pool then returns an error fast (backpressure) instead of hanging requests until client timeout. |

Tune these values based on observed pool metrics (see Feature 11 `/metrics`, which exposes sqlx pool gauges), not guesswork.


### Health endpoint

`GET /health` is public (no auth):

| Request | Meaning |
|---------|---------|
| `GET /health` | **Liveness**: always `200 OK` while the process runs. Orchestrators use this to decide restarts. |
| `GET /health?ready=true` | **Readiness**: runs `SELECT 1` against Postgres with a ~2s timeout; `200 OK` if reachable, `503 Service Unavailable` otherwise. Use this before routing traffic (e.g. during rollouts). |

The Dockerfile `HEALTHCHECK` curls `/health` every 30s.

**Proxying notes:**
- nginx: proxy `/health` like any other route but ensure it is not cached (`proxy_cache off` / excluded from cache rules).
- Cloudflare: set caching level to "Bypass Cache" for `/health*` (Cache Rules → URL path starts with `/health`). Caching health responses hides real outages from orchestrators.
- Behind trusted proxies, readiness reflects the API→DB leg only; end-to-end checks belong at the load balancer/proxy level.

### Metrics endpoint

`GET /metrics` is public (no auth) and exposes Prometheus-format metrics:

| Metric | Type | Description |
|--------|------|-------------|
| `http_requests_total{method,route,status}` | counter | Requests per route pattern/method/status |
| `http_request_duration_seconds` | histogram | Request latency per route |
| `in_flight_requests` | gauge | Currently active requests |
| `db_pool_size` / `db_pool_idle` / `db_pool_waiting_tasks` | gauges | sqlx pool state, polled live from `PgPool` on each scrape |

The pool gauges are what you watch to decide `database_pool.*` tuning values (see above).

**Access restriction:** `/metrics` has no application-level auth because metric labels (routes, status codes) can leak operational detail. Restrict it at nginx — allow internal network/scrapers only, deny everything else:

```nginx
location = /metrics {
    # adjust to your internal subnet / prometheus sidecar address
    allow 10.0.0.0/8;
    allow 172.16.0.0/12;
    allow 192.168.0.0/16;
    deny all;
    proxy_pass http://api_upstream;
}
```

**Scrape interval:** 15–30s is plenty for this workload (Prometheus default is 15s). Do not scrape more often than ~5s — every scrape serializes all registered metrics.

**Caching:**
- nginx: do not cache `/metrics` (`proxy_cache off` / exclude from cache rules); stale metrics hide current saturation.
- Cloudflare: set caching level to "Bypass Cache" for URL path starting with `/metrics` (Cache Rules), same as `/health`.

## Test Docker image locally

See _local_Dockerfile.sh_.  
Useful to check the Dockerfile quickly.  

 
## Deploy

Deploy is executed with a GitHub action that launch a script on a private server.  
The script execution is allowed by a SSH Restricted permission key.  
The description and the procedure is not part of this project.  

### Investigate deploy failure

When the deploy script fails it is possible to look at the Docker container log to see why it fails to start.  
```sh
# get the id of the failed stack
docker stack ps portfolio-api
stack_id=$(docker stack ps portfolio-api --format "{{.ID}} {{.CurrentState}} {{.Error}}" | grep Failed | awk '{print $1}')
echo "stack_id=$stack_id"

container_id=$(docker inspect $stack_id --format '{{.Status.ContainerStatus.ContainerID}}')
echo "container_id=$container_id"

docker logs $container_id

#docker inspect -it $container_id sh
```


## Known Issues

- Debug warning about LLDB not able to debug
  > The LLDB warnings about missing Rust plugins are normal on Windows and do not affect your app's runtime, but they limit debugging features.
