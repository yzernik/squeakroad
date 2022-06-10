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
import json
import logging
import os

import requests
from bitcoin.core import CBlockHeader

from squeaknode.bitcoin.bitcoin_client import BitcoinClient
from squeaknode.bitcoin.block_info import BlockInfo
from squeaknode.bitcoin.exception import BitcoinRequestError

logger = logging.getLogger(__name__)


class BitcoinCoreClient(BitcoinClient):
    """Access a bitcoin daemon using RPC."""

    def __init__(
        self,
        host: str,
        port: int,
        rpc_user: str,
        rpc_password: str,
        use_ssl: bool,
        ssl_cert: str,
    ) -> None:
        protocol = "https" if use_ssl else "http"
        self.url = f"{protocol}://{rpc_user}:{rpc_password}@{host}:{port}"
        self.headers = {"content-type": "application/json"}
        s = requests.Session()
        if ssl_cert:
            os.environ["SSL_CERT_FILE"] = ssl_cert
            os.environ["REQUESTS_CA_BUNDLE"] = ssl_cert
            s.cert = ssl_cert

    def get_best_block_info(self) -> BlockInfo:
        block_height = self.get_block_count()
        logger.debug("Best block height: {}".format(block_height))
        return self.get_block_info_by_height(block_height)

    def get_block_info_by_height(self, block_height: int) -> BlockInfo:
        block_hash = self.get_block_hash(block_height)
        block_header = self.get_block_header(block_hash)
        return BlockInfo(block_height, block_hash, block_header)

    def get_block_count(self) -> int:
        payload = {
            "method": "getblockcount",
            "params": [],
            "jsonrpc": "2.0",
            "id": 0,
        }
        json_response = self.make_request(payload)
        result = json_response["result"]
        return int(result)

    def get_block_hash(self, block_height: int) -> bytes:
        payload = {
            "method": "getblockhash",
            "params": [block_height],
            "jsonrpc": "2.0",
            "id": 0,
        }
        json_response = self.make_request(payload)
        result = json_response["result"]
        return bytes.fromhex(result)

    def get_block_header(self, block_hash: bytes, verbose: bool = False) -> CBlockHeader:
        payload = {
            "method": "getblockheader",
            "params": [block_hash.hex(), verbose],
            "jsonrpc": "2.0",
            "id": 0,
        }
        json_response = self.make_request(payload)
        result = json_response["result"]
        header_bytes = bytes.fromhex(result)
        return CBlockHeader.deserialize(header_bytes)

    def make_request(self, payload: dict) -> dict:
        try:
            response = requests.post(
                self.url,
                data=json.dumps(payload),
                headers=self.headers,
            )
            response.raise_for_status()
        except requests.exceptions.HTTPError as errh:
            raise BitcoinRequestError(errh)
        except requests.exceptions.ConnectionError as errc:
            raise BitcoinRequestError(errc)
        except requests.exceptions.Timeout as errt:
            raise BitcoinRequestError(errt)
        except requests.exceptions.RequestException as err:
            raise BitcoinRequestError(err)

        return response.json()
