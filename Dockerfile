FROM rust as build
WORKDIR /app
COPY Cargo.lock Cargo.toml ./
RUN mkdir src && echo "// dummy file" > src/lib.rs && cargo fetch --locked && rm src/lib.rs
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt update && apt install libgtk-3-dev libnotify-dev libgconf-2-4 libnss3 libxss1 libasound2 -y
WORKDIR /app
COPY --from=build /app/target/release/paperdash-backend /app/
RUN chmod +x /app/paperdash-backend

ENV ROCKET_PORT 80
ENV ROCKET_ADDRESS 0.0.0.0

EXPOSE 80

ENTRYPOINT [ "/bin/bash", "-l", "-c" ]
CMD ["/app/paperdash-backend"]
