
cp -r ../proto src/

(cd src && \
    protoc -I=. proto/squeak_admin.proto proto/lnd.proto \
	   --js_out=import_style=commonjs:. \
	   --grpc-web_out=import_style=commonjs,mode=grpcwebtext:.)

./pb_disable-eslint.sh src/proto/
