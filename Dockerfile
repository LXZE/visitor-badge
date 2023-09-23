# 0

# 1. Build
FROM rust:1.70 as build

WORKDIR /usr/src/visitor-badge

RUN apt-get update && apt-get install libsqlite3-dev -y
RUN cargo install diesel_cli --no-default-features --features sqlite

COPY . .
COPY .env .env
COPY .env.docker .env.docker

RUN diesel setup
RUN diesel migration run

RUN cargo install --path .

# 2. Deploy
FROM gcr.io/distroless/cc-debian11
# FROM alpine:latest
ARG ARCH=x86_64
COPY --from=build /bin/sh /bin/sh
COPY --from=build /usr/lib/${ARCH}-linux-gnu/libsqlite3.so* /usr/lib/${ARCH}-linux-gnu/
COPY --from=build /usr/local/cargo/bin/visitor-badge /usr/local/bin/visitor-badge
COPY --from=build /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY --from=build --chmod=664 /usr/src/visitor-badge/sqlite.db /tmp/sqlite.db
COPY --from=build /usr/src/visitor-badge/ /app/
COPY --from=build /usr/src/visitor-badge/.env.docker /app/.env
# COPY --from=build /usr/src/visitor-badge/src/fonts/DejaVuSans.ttf /src/fonts/DejaVuSans.ttf
CMD ["visitor-badge"]
