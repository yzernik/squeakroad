all: clean build

clean:
	rm -rf build node_modules

rundev:	gen-protos
	npm install
	REACT_APP_DEV_MODE_ENABLED=true REACT_APP_SERVER_PORT=12994 npm start

build:  gen-protos
	npm install
	npm run build
	rm -rf ../squeaknode/admin/webapp/static/build
	cp -r build/ ../squeaknode/admin/webapp/static

lint:	gen-protos
	npm install
	eslint src/ --fix

gen-protos:
	./build-protos.sh

.PHONY: all clean rundev build gen-protos
