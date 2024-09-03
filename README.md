# Worship Viewer

**Worship Viewer** is an app for managing and displaying digital sheet music.
It allows users to import entire music books as collections, providing a digital table of contents with corresponding metadata that is searchable.

## Usage

The app is comprised of a backend ([actix-web](https://actix.rs/)) and a frontend ([yew](https://yew.rs/)), both of which are built together into a Docker image.
The backend communicates with a [SurrealDB](https://surrealdb.com/) database. 
Worship Viewer is designed for multi-tenant operation but does not handle authentication internally. 
Instead, it relies on an external authentication proxy like [Proxauth](https://github.com/xilefmusics/proxauth) to set the `X-Remote-User` header.

### Production

For production use, it is recommended to deploy the Docker image, which is pre-built on [DockerHub](https://hub.docker.com/repository/docker/xilefmusics/worship-viewer) for each release.
Additionally, the [SurrealDB](https://hub.docker.com/r/surrealdb/surrealdb) and [Proxauth](https://hub.docker.com/r/xilefmusics/proxauth) Docker images can be used to set up a fully functional system. 

You can find an example configuration in the [docker-compose.yaml](https://github.com/xilefmusics/worship_viewer/blob/main/docker-compose.yaml) file.
To start the application, run:

```bash
docker compose up
```

After starting, you can log in at `localhost:8080` using the credentials:

- Username: `test`
- Password: `test`

This login is valid for 24 hours.
If you encounter an error message after this period, it means you haven't been automatically logged out.
You can log out manually at `localhost:8080/logout`, which will redirect you to the login page for re-authentication.

### Development

For development purposes, the frontend and backend should be started separately to enable automatic rebuilding of both components.
Proxauth should be configured to forward requests appropriately: frontend requests to the frontend and backend requests to the backend.

An example configuration is provided in the [proxauth-config.yaml](https://github.com/xilefmusics/worship_viewer/blob/main/proxauth-config.yaml) file.

Ensure that SurrealDB (version 1.5.3) and Proxauth (version 0.1.0) are installed as dependencies. 
In addition, the two crates [fancy-yew](https://github.com/xilefmusics/fancy_yew) and [fancy-surreal](https://github.com/xilefmusics/fancy_surreal) are required, which unfortunately are not yet on [crates.io](https://crates.io/) and are therefore needed in parallel to the money-app repository.
More detailed information can be found in the [Dockerfile](https://github.com/xilefmusics/money-app/blob/main/Dockerfile).

Once all dependencies are installed, start the four components using the following commands:

``` bash
surreal start --log debug --user root --pass root memory --allow-scripting
cd backend && cargo watch -cqx run
cd frontend && trunk serve
CONFIG_FILE="./proxauth-config.yaml" proxauth
```

You can then log in at `localhost:8081` using the same credentials:

- Username: `test`
- Password: `test`

This login is also valid for 24 hours.
If you see an error message after this time, manually log out at `localhost:8081/logout` and then log back in.

## License

[![GPL-3.0](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
