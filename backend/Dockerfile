# Multi-stage build for the finviz-api server.
# Build context: the `backend/` directory.
FROM rust:1.94-slim AS build
WORKDIR /app
COPY . .
RUN cargo build --release -p finviz-api

FROM debian:trixie-slim
RUN useradd --create-home --uid 10001 app
COPY --from=build /app/target/release/finviz-api /usr/local/bin/finviz-api
USER app
ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
CMD ["finviz-api"]
