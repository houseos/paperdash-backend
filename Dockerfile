FROM rust as build
WORKDIR /app
COPY Cargo.lock Cargo.toml ./
COPY src ./src

RUN cargo fetch --locked
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build /app/target/release/paperdash-backend /app/
RUN chmod +x /app/paperdash-backend

RUN apt-get update && apt-get install wget
RUN wget -q -O - https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add -
RUN sh -c 'echo "deb http://dl.google.com/linux/chrome/deb/ stable main" >> /etc/apt/sources.list.d/google.list'
RUN apt-get update
RUN apt --fix-broken install
RUN apt-get install google-chrome-stable -y

ENV ROCKET_PORT 80
ENV ROCKET_ADDRESS 0.0.0.0

EXPOSE 80

ENTRYPOINT [ "/bin/bash", "-l", "-c" ]
CMD ["/app/paperdash-backend"]
