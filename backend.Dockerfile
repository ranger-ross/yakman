FROM rust:latest

# create a new empty shell project
RUN USER=root cargo new --bin app
WORKDIR /app
RUN USER=root cargo new --bin backend
RUN USER=root cargo new --bin core
RUN USER=root cargo new --bin frontend

# copy over manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./backend/Cargo.toml ./backend/Cargo.toml
COPY ./core/Cargo.toml ./core/Cargo.toml

# Fetch dependencies
RUN cargo build --release
RUN rm ./src/*.rs
RUN rm -rf ./backend/src
RUN rm -rf ./core/src

# copy source
COPY ./src ./src
COPY ./backend ./backend
COPY ./core ./core
COPY ./frontend ./frontend

RUN rm ./target/release/yak-man-backend*
RUN cargo build --release


CMD ["./target/release/yak-man-backend"]