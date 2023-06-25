FROM rust:1.70-alpine
LABEL authors="Hugh Mandalidis"

WORKDIR /usr/src/app

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# cache dependencies
RUN mkdir src/
RUN echo "fn main() {println!(\"broken build :(\")})" > src/main.rs
RUN cargo build --release
RUN rm -f src/*.rs

# copy source code to container
COPY ./src ./src

# rebuild with cached deps
RUN cargo build --release

CMD ["./target/release/leetcode_bot"]
