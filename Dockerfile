FROM rustlang/rust as build
WORKDIR /app
COPY src Cargo.lock Cargo.toml .

RUN cargo build --release


FROM alpine
WORKDIR /app
COPY --from=build /app/target/release/paperdash-backend .

ENV ROCKET_PORT 80
ENV ROCKET_ADDRESS 0.0.0.0

EXPOSE 80

CMD ["/app/paperdash-backend"]
