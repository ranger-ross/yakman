FROM rust:latest as builder

# Create a new empty shell project
RUN USER=root cargo new --bin backend

WORKDIR /backend

# Copy Cargo files
COPY ./backend/Cargo.toml Cargo.toml
COPY ./backend/Cargo.lock Cargo.lock


# Fetch dependencies
RUN cargo build --release
# Remove temporary binary + source code
RUN cargo clean --release -p yak-man-backend
RUN rm -rf src

# Copy source
COPY ./backend/src ./src

RUN cargo build --release


CMD ["/backend/target/release/yak-man-backend"]

FROM debian:bookworm-slim
RUN apt update
RUN apt install -y ca-certificates
COPY --from=builder /backend/target/release/yak-man-backend /usr/local/bin/yak-man-backend
CMD ["yak-man-backend"]
