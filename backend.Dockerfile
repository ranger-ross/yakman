FROM rust:latest as builder

# create a new empty shell project
RUN USER=root cargo new --bin backend

WORKDIR /backend


# Copy Cargo files
COPY ./backend/Cargo.toml Cargo.toml
COPY ./backend/Cargo.lock Cargo.lock


# Fetch dependencies
RUN cargo build --release
RUN rm -rf src
# TODO: This command makes the build slight slower,
#       but for some reason Cargo will not compile the project without it.
RUN rm -rf ./target/release/build
RUN rm -rf ./target/release/yak-man-backend*

# copy source
COPY ./backend/src ./src

RUN cargo build --release


CMD ["/backend/target/release/yak-man-backend"]

FROM debian:bookworm-slim
RUN apt update
RUN apt install -y ca-certificates
COPY --from=builder /backend/target/release/yak-man-backend /usr/local/bin/yak-man-backend
CMD ["yak-man-backend"]
