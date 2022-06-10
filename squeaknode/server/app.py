# MIT License
#
# Copyright (c) 2020 Jonathan Zernik
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
import logging
import os
import threading

from flask import Flask
from flask import jsonify
from flask import request
from werkzeug.serving import make_server

from squeaknode.server.squeak_peer_server_handler import NotFoundError
from squeaknode.server.squeak_peer_server_handler import PaymentRequiredError

logger = logging.getLogger(__name__)


def create_app(handler):
    # create and configure the app
    logger.debug("Starting flask app from directory: {}".format(os.getcwd()))
    app = Flask(
        __name__,
        static_url_path="/",
        static_folder="static/build",
    )
    app.config.from_mapping(
        SECRET_KEY="dev",
    )
    logger.debug("Starting flask with app.root_path: {}".format(app.root_path))
    logger.debug("Files in root path: {}".format(os.listdir(app.root_path)))

    @app.route("/")
    def index():
        return "Hello, Index!"

    @app.route("/hello")
    def hello_world():
        return "Hello, World!"

    @app.route('/squeak/<hash>')
    def squeak(hash):
        try:
            squeak_bytes = handler.handle_get_squeak_bytes(hash)
        except NotFoundError:
            return "Not found", 404
        return squeak_bytes

    @app.route('/secretkey/<hash>')
    def secret_key(hash):
        try:
            secret_key_bytes = handler.handle_get_secret_key(hash)
        except NotFoundError:
            return "Not found", 404
        except PaymentRequiredError:
            return "Payment required", 402
        return secret_key_bytes

    @app.route('/offer/<hash>')
    def offer(hash):
        client_host = request.remote_addr
        client_port = request.environ.get('REMOTE_PORT') or 0
        try:
            offer = handler.handle_get_offer(hash, client_host, client_port)
        except NotFoundError:
            return "Not found", 404
        return jsonify({
            'squeak_hash': offer.squeak_hash.hex(),
            'nonce': offer.nonce.hex(),
            'payment_request': offer.payment_request,
            'host': offer.host,
            'port': offer.port,
        })

    @app.route("/lookup")
    def lookup():
        min_block = request.args.get('minblock')
        max_block = request.args.get('maxblock')
        pubkeys = request.args.getlist('pubkeys')
        if len(pubkeys) == 0:
            squeak_hashes = []
        else:
            squeak_hashes = handler.handle_lookup_squeaks(
                pubkeys,
                min_block,
                max_block,
            )
        squeak_hashes_str = [
            squeak_hash.hex()
            for squeak_hash in squeak_hashes
        ]
        return jsonify(squeak_hashes_str)

    # @sock.route('/echo')
    # def echo(ws):
    #     count = 0
    #     while True:
    #         data = f'hello_{count}'
    #         logger.info(data)
    #         ws.send(data)
    #         count += 1
    #         import time
    #         time.sleep(5)

    # @sock.route('/subscribetimeline')
    # def subscribe_timeline(ws):
    #     logger.info("Getting lookup route.")
    #     min_block = request.args.get('minblock')
    #     max_block = request.args.get('maxblock')
    #     pubkeys = request.args.getlist('pubkeys')
    #     logger.info("Hello, lookup! Min block {}, Max block {}, pubkeys: {}".format(
    #         min_block,
    #         max_block,
    #         pubkeys,
    #     ))
    #     # TODO: This special case should not need to be handled here.
    #     if len(pubkeys) == 0:
    #         squeak_hashes = []
    #     else:
    #         squeak_hashes = handler.handle_lookup_squeaks(
    #             pubkeys,
    #             min_block,
    #             max_block,
    #         )
    #     for squeak_hash in squeak_hashes:
    #         ws.send(squeak_hash.hex())

    return app


class SqueakPeerWebServer:
    def __init__(
        self,
        host,
        port,
        handler,
    ):
        self.host = host
        self.port = port
        self.app = create_app(handler)
        self.server = None

    def get_app(self):
        return self.app

    def start(self):
        self.server = make_server(
            self.host,
            self.port,
            self.get_app(),
            threaded=True,
            ssl_context=None,
        )

        logger.info("Starting SqueakPeerWebServer...")
        threading.Thread(
            target=self.server.serve_forever,
        ).start()

    def stop(self):
        if self.server is None:
            return
        logger.info("Stopping SqueakPeerWebServer....")
        self.server.shutdown()
        logger.info("Stopped SqueakPeerWebServer.")
