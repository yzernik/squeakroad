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

from squeak.core.keys import SqueakPublicKey

from squeaknode.core.offer import Offer
from squeaknode.core.peer_address import Network
from squeaknode.core.peer_address import PeerAddress
from squeaknode.node.squeak_controller import SqueakController

logger = logging.getLogger(__name__)


class PaymentRequiredError(Exception):
    pass


class NotFoundError(Exception):
    pass


class SqueakPeerServerHandler(object):
    """Handles peer server commands."""

    def __init__(
            self,
            squeak_controller: SqueakController,
            node_settings,
            config,
    ):
        self.squeak_controller = squeak_controller
        self.node_settings = node_settings
        self.config = config

    def handle_get_squeak_bytes(self, squeak_hash_str) -> bytes:
        """Return a tuple with the squeak type and the squeak bytes.

        Returns: bytes
        """
        squeak_hash = bytes.fromhex(squeak_hash_str)
        squeak = self.squeak_controller.get_squeak(squeak_hash)
        if not squeak:
            raise NotFoundError()
        return squeak.serialize()

    def handle_get_secret_key(self, squeak_hash_str) -> bytes:
        squeak_hash = bytes.fromhex(squeak_hash_str)
        price_msat = self.squeak_controller.get_sell_price_msat()
        if price_msat > 0:
            raise PaymentRequiredError()
        secret_key = self.squeak_controller.get_squeak_secret_key(squeak_hash)
        if not secret_key:
            raise NotFoundError()
        return secret_key

    def handle_get_offer(self, squeak_hash_str, client_host, client_port) -> Offer:
        squeak_hash = bytes.fromhex(squeak_hash_str)
        client_addr = PeerAddress(
            network=Network.IPV4,
            host=client_host,
            port=client_port,
        )
        offer = self.squeak_controller.get_packaged_offer(
            squeak_hash,
            client_addr,
        )
        if not offer:
            raise NotFoundError()
        return offer

    def handle_lookup_squeaks(
            self,
            pubkey_strs: List[str],
            min_block: Optional[int],
            max_block: Optional[int],
    ) -> List[bytes]:
        pubkeys = [
            SqueakPublicKey.from_bytes(bytes.fromhex(pubkey_str))
            for pubkey_str in pubkey_strs
        ]
        # Add separate endpoint for replies.
        # reply_to_hash = interest.hashReplySqk if interest.hashReplySqk != EMPTY_HASH else None
        return self.squeak_controller.lookup_squeaks(
            pubkeys,
            min_block,
            max_block,
            None,
        )
