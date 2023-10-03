FROM rust:slim-bookworm as build
WORKDIR /app
COPY Cargo.lock Cargo.toml ./
COPY src ./src

RUN cargo fetch --locked
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build /app/target/release/paperdash-backend /app/
RUN chmod +x /app/paperdash-backend

ENV ROCKET_PORT 80
ENV ROCKET_ADDRESS 0.0.0.0

EXPOSE 80

ENTRYPOINT [ "/bin/bash", "-l", "-c" ]
CMD ["/app/paperdash-backend"]
