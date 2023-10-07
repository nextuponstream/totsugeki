# Combine both axum app and vue app
# build with docker:
# docker build -t to-app .
# TODO still 98mb send in docker build context...
# need slimer image (1.5G)

# docker run -d -p 3000:3000 to-app

FROM rust:1.73.0 as builder
COPY . .
RUN cargo build --release --package tournament-organiser-api 

# ---
FROM node:18 as static
COPY . .
RUN npm --prefix tournament-organiser-web install
RUN npm --prefix tournament-organiser-web run build

# ---
FROM rust:1.73.0 as runtime
ENV BUILD_PATH_TOURNAMENT_ORGANISER_WEB dist
ENV DOCKER_BUILD 1
COPY --from=static tournament-organiser-web/dist dist
COPY --from=builder /target/release/tournament-organiser-api tournament-organiser-api
ENTRYPOINT ["./tournament-organiser-api"]