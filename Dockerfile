FROM rust:1.62.0-buster AS builder

COPY . ./

RUN cargo install --path .

FROM debian:buster-slim

RUN apt-get update && \
    apt-get -y install openssl

COPY --from=builder /usr/local/cargo/bin/squeakroad /usr/local/bin/squeakroad
COPY ./static /static
COPY ./templates /templates

COPY "entrypoint.sh" .
RUN chmod +x entrypoint.sh

CMD ["./entrypoint.sh"]
