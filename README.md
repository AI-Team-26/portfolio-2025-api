# Portfolio 2025 - API

[![Deploy](https://github.com/alex-piccione/portfolio-2025-api/actions/workflows/deploy.yml/badge.svg)](https://github.com/alex-piccione/portfolio-2025-api/actions/workflows/deploy.yml)

Portfolio: Software to Keep note of your assets that you own in different insitutions.   
This project is the back-end API for Portfolo.  

## Technology

**Rust**: code language used for ythis API
**Axum**: Rust library for API service.  
**Sqlx**: Rust library to interact with database (compile-time schema checks).
**Docker**: Containerization software used to run the services instances
**Postgres**: Database

## Setup

- _.env_ file: create a .env file on the root, see _.env_example_ as reference.
- convert migrations file to Linux format, if on Windows machine ( see README inside /migrations folder)

### Environment variables

| Variable | Used by | Purpose |
|---|---|---|
| `DATABASE_URL` | SQLx CLI (`cargo sqlx prepare`, migrations) | Connection string for the local Postgres container |
| `CONFIGURATION_FILE` | Application runtime | Optional path to the configuration JSON (template: `src/configuration_example.json`) |
| `APP_*` | Application runtime | Environment overrides, nested keys use `__`, e.g. `APP_SERVER_PORT`, `APP_SECRETS__COINGECKO_API_KEY` |
| `RUST_LOG` | _tracing-subscriber_ | Log level filter, e.g. `info` or `your_crate=debug,tower=info` |

Runtime settings (e.g. `server_port`) are resolved with layered precedence: **environment variables (`APP_*`) > configuration file (`CONFIGURATION_FILE`) > built-in defaults**. The configuration file is optional; only `database_connection_string` has no default.


## Development

_rust-analyzer_ continuously checks the code and highlights issues, `cargo build` will compile the project with the list of errors too.

### SQLx

SQLx is set to verify the database entity and need to be able to access the database.  
It uses the **DATABASE_URL** variable set in the _.env_ file, it points to a local Docker container with Postgres.  
`cargo sqlx prepare`.  
Note: `cargo sqlx prepare` needs a reachable database via `DATABASE_URL`; CI instead uses the committed `.sqlx` cache with `SQLX_OFFLINE=true` (no DB required).
  
Refer to the [SQLx](src/repositories/SQLx.md) readme.

### DateTime
Rust standard library does not have Date or Datetime types (!).  
A modern approach is to use _chrono_ but also _sqlx_ offer it.  
To keep it simple, I'll use sqlx types. 
I'll use _OffsetDateTime_ of SQLx (always as UTC).  
I created UtcDateTime custom type... I tried to use only std types like Duration... rubbish!  
In the end only _chrono_ has a clear and short "Utc::now()" function and allows "<", ">" and "=" operators.  

## Run locally

### Debug Localhost
VS Code launch (_launch.json_) is set to run the app locally.  

Sometime the process is still running despite terminal and debug are closed.  
Run this to find the process PID (<process>.exe):  
```sh
tasklist | findstr portfolio`
```
Run this to delete it:  
```powershell or CMD
taskkill /PID <PID> /F
```

### Docker
See [devop/README.md](devop/README.md) for instructions to run the api and database on local Docker.

## Deploy

### On private server

"distroless" Dockerfile advantages:
- Minimal attack surface (no shell, package manager, etc.)
- Only contains your application and minimal runtime
- Regularly updated by Google
- Industry standard for production containers

The deploy is executed running a script on a private server.  
See [devop/README.md](devop/README.md#Deploy) for how to configure the script.


## Logging

I use _tracing_ and _tracing-subscriber_.  
Use the environment variable _RUST_LOG_ (`RUST_LOG=info` or `RUST_LOG=your_crate=debug,tower=info`).
There are macros in _logging.rs_ to facilitate it.  

TODO:  
- logs with macro
- setup log_level in production
- Grafana
- Loki


## Tips

### Read environment variable

```rust
// read the port from environment variable or use a default
let port = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse::<u16>()
    .expect("Failed to parse PORT environment variable as a number");
```

### static mut
_static mut_ in Rust is ... not possible.  
Not without _unsafe_.  
In other languages you have thread-safe collections, but not in Rust standard library.
You can play around and write cumbersome code with LazyLock, RwLock and Mutex... that's it.
Otherwise, third party library. I choose "Dashmap". There is not an equivalent of Dashmap for collections! 

### Read local file
(used previously to read configuration)

```rust
let config_file = match std::fs::exists(CONFIGURATION_FILE) {
    Ok(true) => { 
        println!("Using configuration file '{}'.", CONFIGURATION_FILE); 
        String::from(CONFIGURATION_FILE)
    },
    Ok(false) => { 
        println!("Configuration file '{}' not found, using CONFIGURATION_FILE environment variable.", CONFIGURATION_FILE); 

    },
    Err(e) => panic!("Failed to check for local configuration file '{}': {}", CONFIGURATION_FILE, e),
};
```
