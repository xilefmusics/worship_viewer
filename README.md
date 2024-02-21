# Worship Viewer

he Worship Viewer is an app for managing and displaying digital sheet music. 
Entire music books can be imported as collections and have a digital table of contents with the corresponding metadata and can be searched.

## Usage

The app consists of a backend ([actix-web](https://actix.rs/)) and a frontend ([yew](https://yew.rs/)), which are both built together in a Docker image.
The backend communicates with a [SurrealDB](https://surrealdb.com/) database.
The app is designed for multi-tenant operation, but does not take care of authentication itself. 
Instead, it relies on an authentication proxy such as [Proxauth](https://github.com/xilefmusics/proxauth), which sets the `X-Remote-User` header.

### Production

For productive operation, it is recommended to use the Docker Image, which is already built on [DockerHub](https://hub.docker.com/repository/docker/xilefmusics/worship-viewer) for the respective releases.
In addition, the [SurrealDB](https://hub.docker.com/r/surrealdb/surrealdb) and [Proxauth](https://hub.docker.com/r/xilefmusics/proxauth) image can be used to create a fully functional overall system.
An example configuration can be seen in [docker-compose.yaml](https://github.com/xilefmusics/worship-viewer/blob/main/docker-compose.yaml) and started via:

```bash
docker compose up
```

Now you can login at `localhost:8080` using the user `test` and the password `test`.
This login is valid for 24h.
If you see an error message after this time, you have not been logged out automatically and can log out via `localhost:8080/logout`. 
You will then be redirected to the login page and can log in again.

### Development

As a development setup, the frontend and the backend must be started separately to ensure an auto rebuild for both components.
Proxauth can now be configured to forward frontend requests to the frontend and backend requests to the backend.
An example configuration for this can be found in [proxauth-config.yaml](https://github.com/xilefmusics/worship-viewer/blob/main/proxauth-config.yaml).

Surrealdb (1.0.0) and proxauth (0.1.0) must be installed as dependencies.
More detailed information can be found in the [Dockerfile](https://github.com/xilefmusics/worship-viewer/blob/main/Dockerfile).

Once all dependencies have been installed, the four components can be started using the following four commands:

``` bash
surreal start --log debug --user root --pass root memory --allow-scripting
cd backend && cargo watch -cqx run
cd frontend && trunk serve
CONFIG_FILE="./proxauth-config.yaml" proxauth
```

Now you can login at `localhost:8081` using the user `test` and the password `test`.
This login is valid for 24h.
If you see an error message after this time, you have not been logged out automatically and can log out via `localhost:8080/logout`. 
You will then be redirected to the login page and can log in again.

## License

[![GPL-3.0](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
