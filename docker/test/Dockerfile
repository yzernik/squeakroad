FROM python:3.8-slim

COPY requirements-itest.txt /

RUN pip3 install -r requirements-itest.txt

WORKDIR /app

COPY proto ./proto
COPY itests/test.sh ./
COPY itests/tests ./tests

RUN python3 -m grpc_tools.protoc --proto_path=. --python_out=. --grpc_python_out=. \
	proto/lnd.proto \
	proto/squeak_admin.proto

RUN chmod +x test.sh
CMD ["bash", "test.sh"]
