# Combine both axum app and vue app
# build with docker:
# docker build -t to-app .
# TODO still 98mb send in docker build context...

# docker run -d -p 8080:8080 to-app

# build rust binary
# alpine and slim-bullseye does not work
FROM rust:1.73.0-bullseye as builder
ENV SQLX_OFFLINE=true
COPY . .
RUN cargo build --release --package tournament-organiser-api 

# ---
FROM node:18-alpine as static
# TODO make it configurable
ENV VITE_API_URL https://totsugekitest.fly.dev/api
COPY . .
RUN npm --prefix tournament-organiser-web install
RUN npm --prefix tournament-organiser-web run build

# ---
# size: around 93mb, good enough
FROM debian:bullseye-slim as runtime
ENV BUILD_PATH_TOURNAMENT_ORGANISER_WEB dist
ENV DOCKER_BUILD 1
COPY --from=static tournament-organiser-web/dist dist
COPY --from=builder /target/release/tournament-organiser-api tournament-organiser-api
ENTRYPOINT ["./tournament-organiser-api"]