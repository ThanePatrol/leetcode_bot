FROM rust:1.70-buster

WORKDIR /usr/app

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./src ./src

RUN cargo build --release

CMD ["./target/release/leetcode_bot"]
