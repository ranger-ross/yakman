FROM rust:latest as builder

# create a new empty shell project
RUN USER=root cargo new --bin app
WORKDIR /app
RUN USER=root cargo new --bin backend
RUN USER=root cargo new --bin core
RUN USER=root cargo new --bin legacy_frontend

# Copy Cargo files
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
COPY ./frontend ./legacy_frontend

RUN rm ./target/release/yak-man-backend*
RUN cargo build --release


FROM debian:bullseye-slim
COPY --from=builder /app/target/release/yak-man-backend /usr/local/bin/yak-man-backend
CMD ["yak-man-backend"]
