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
from typing import List
from typing import Optional

import requests
from squeak.core import CBaseSqueak
from squeak.core import CResqueak
from squeak.core import CSqueak
from squeak.core.keys import SqueakPublicKey

from squeaknode.core.offer import Offer
from squeaknode.core.peer_address import Network
from squeaknode.core.squeak_peer import SqueakPeer

logger = logging.getLogger(__name__)


REQUEST_TIMEOUT_S = 10


class PeerClient:

    def __init__(
            self,
            peer: SqueakPeer,
            proxy_host: Optional[str],
            proxy_port: Optional[int],
    ):
        self.peer = peer
        self.proxy_host = proxy_host
        self.proxy_port = proxy_port
        self.base_url = f"http://{peer.address.host}:{peer.address.port}"
        self.proxies = {}
        if peer.address.network == Network.TORV3 and \
           proxy_host is not None and \
           proxy_port is not None:
            self.proxies = {
                "http": f'socks5h://{proxy_host}:{proxy_port}',
            }
        logger.debug("Using base url: {}".format(self.base_url))
        logger.debug("Using proxies: {}".format(self.proxies))

    def lookup(
            self,
            min_block: int,
            max_block: int,
            pubkeys: List[SqueakPublicKey],
    ) -> List[bytes]:
        pubkeys_str = [
            pubkey.to_bytes().hex()
            for pubkey in pubkeys
        ]
        payload = {
            'minblock': min_block,
            'maxblock': max_block,
            'pubkeys': pubkeys_str,
        }
        url = f"{self.base_url}/lookup"
        r = requests.get(  # type: ignore
            url,
            params=payload,  # type: ignore
            proxies=self.proxies,
            timeout=REQUEST_TIMEOUT_S,
        )
        squeak_hashes_str = r.json()
        return [
            bytes.fromhex(squeak_hash_str)
            for squeak_hash_str in squeak_hashes_str
        ]

    def get_squeak(self, squeak_hash: bytes) -> Optional[CBaseSqueak]:
        squeak_hash_str = squeak_hash.hex()
        url = f"{self.base_url}/squeak/{squeak_hash_str}"
        r = requests.get(
            url,
            proxies=self.proxies,
            timeout=REQUEST_TIMEOUT_S,
        )
        if r.status_code != requests.codes.ok:
            return None
        squeak_bytes = r.content
        try:
            return CSqueak.deserialize(squeak_bytes)
        except Exception:
            pass
        try:
            return CResqueak.deserialize(squeak_bytes)
        except Exception:
            pass
        return None

    def get_secret_key(self, squeak_hash: bytes) -> Optional[bytes]:
        squeak_hash_str = squeak_hash.hex()
        url = f"{self.base_url}/secretkey/{squeak_hash_str}"
        r = requests.get(
            url,
            proxies=self.proxies,
            timeout=REQUEST_TIMEOUT_S,
        )
        if r.status_code != requests.codes.ok:
            return None
        secret_key = r.content
        return secret_key

    def get_offer(self, squeak_hash: bytes) -> Optional[Offer]:
        squeak_hash_str = squeak_hash.hex()
        url = f"{self.base_url}/offer/{squeak_hash_str}"
        r = requests.get(
            url,
            proxies=self.proxies,
            timeout=REQUEST_TIMEOUT_S,
        )
        if r.status_code != requests.codes.ok:
            return None
        offer_json = r.json()
        offer = Offer(
            squeak_hash=bytes.fromhex(offer_json['squeak_hash']),
            nonce=bytes.fromhex(offer_json['nonce']),
            payment_request=offer_json['payment_request'],
            host=offer_json['host'],
            port=int(offer_json['port']),
        )
        return offer
