#!/usr/bin/bash

npm --prefix tournament-organiser-web run build \
&& \
cargo run --package tournament-organiser-api