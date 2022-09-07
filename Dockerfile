FROM rust:1.62.0-buster AS builder

RUN DEBIAN_FRONTEND=noninteractive apt-get update && \
	apt-get install -y \
	libgexiv2-dev \
	cmake

COPY . ./

RUN cargo install --path .

FROM debian:buster-slim

RUN DEBIAN_FRONTEND=noninteractive apt-get update && \
	apt-get install -y \
	openssl \
	libgexiv2-dev

RUN DEBIAN_FRONTEND=noninteractive \
  apt-get update && \
  apt-get install ca-certificates && \
  apt-get clean

COPY ./cacert.pem /usr/share/ca-certificates/cacert.pem
RUN chmod 644 /usr/share/ca-certificates/cacert.pem && update-ca-certificates

COPY --from=builder /usr/local/cargo/bin/squeakroad /usr/local/bin/squeakroad
COPY ./static /static
COPY ./templates /templates

COPY "entrypoint.sh" .
RUN chmod +x entrypoint.sh

CMD ["./entrypoint.sh"]
