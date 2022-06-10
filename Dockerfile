FROM python:3.8-slim-buster AS compile-image

WORKDIR /

RUN DEBIAN_FRONTEND=noninteractive apt-get update && \
	apt-get install -y \
	libpq-dev \
	gcc \
	libffi-dev \
	build-essential

RUN python -m venv /opt/venv
# Make sure we use the virtualenv:
ENV PATH="/opt/venv/bin:$PATH"

RUN pip install --upgrade pip

WORKDIR /app

# Copy the source code.
COPY . .

RUN pip install .[postgres]

FROM python:3.8-slim-buster

COPY --from=compile-image /opt/venv /opt/venv

RUN DEBIAN_FRONTEND=noninteractive apt-get update && \
	apt-get install -y libpq-dev

EXPOSE 8555
EXPOSE 18555
EXPOSE 18666
EXPOSE 18777
# Web server
EXPOSE 12994

# Make sure we use the virtualenv:
ENV PATH="/opt/venv/bin:$PATH"

# Copy the entrypoint script.
COPY "entrypoint.sh" .
RUN chmod +x entrypoint.sh

CMD ["./entrypoint.sh"]
