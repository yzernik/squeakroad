# development

### Run in development mode:

#### Background services

- Start the required background services in docker:
	```
	cd docker
	docker-compose build
	docker-compose up
	```

	This will start the following services in containers:
	* bitcoin-core
	* lnd (for the containerized squeaknode)
	* lnd (for the host squeaknode) with ~/.lnd directory mounted.
	* a tor service
	* a tor socks proxy
	* a squeaknode (useful for testing p2p connections from your host machine)

	You can go to http://localhost:12995/ to see the containerized squeaknode.

#### Squeaknode backend

You can also run your own squeaknode on your host machine.

- Install squeaknode:
	```
	python3 -m venv venv
	source venv/bin/activate
	pip install .
	```

- Copy the `~/.lnd` directory from the docker container to your host machine.
	```
	docker cp lnd_client:/root/.lnd/ ~/.lnd
	```

- Run squeaknode with authentication disabled:
	```
	SQUEAKNODE_WEBADMIN_ENABLED=TRUE \
	SQUEAKNODE_WEBADMIN_LOGIN_DISABLED=TRUE \
	SQUEAKNODE_WEBADMIN_ALLOW_CORS=TRUE \
	SQUEAKNODE_NETWORK=testnet \
	squeaknode --config config.ini
	```

#### Squeaknode frontend

- Install `protoc-gen-grpc-web` (https://github.com/grpc/grpc-web#code-generator-plugin)
- Run the frontend react app in dev mode:
	```
	cd frontend
	make rundev
	```
- Lint the frontend code:
	```
	node_modules/.bin/eslint .
	```
	or
	```
	node_modules/.bin/eslint . --fix
	```


#### Connect to the containerized squeaknode

- To open a p2p connection from your host squeaknode to the containerized
squeaknode, use `localhost` as host and `18557` as port.
