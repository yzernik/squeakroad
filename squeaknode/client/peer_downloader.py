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
from abc import ABC
from typing import List
from typing import Optional

from squeak.core.keys import SqueakPublicKey

from squeaknode.client.peer_client import PeerClient
from squeaknode.core.squeak_peer import SqueakPeer
from squeaknode.core.squeaks import get_hash
from squeaknode.node.squeak_store import SqueakStore

logger = logging.getLogger(__name__)


DOWNLOAD_TIMEOUT_S = 10


class PeerDownloader(ABC):

    def __init__(
            self,
            peer: SqueakPeer,
            squeak_store: SqueakStore,
            proxy_host: Optional[str],
            proxy_port: Optional[int],
    ):
        self.peer = peer
        self.proxy_host = proxy_host
        self.proxy_port = proxy_port
        self.client = PeerClient(peer, proxy_host, proxy_port)
        self.squeak_store = squeak_store

    def download_interest_range(
            self,
            min_block: int,
            max_block: int,
            pubkeys: List[SqueakPublicKey],
    ) -> None:
        squeak_hashes = self.client.lookup(
            min_block,
            max_block,
            pubkeys,
        )
        for squeak_hash in squeak_hashes:
            try:
                self.download_squeak(
                    squeak_hash,
                    min_block,
                    max_block,
                    pubkeys,
                )
            except Exception:
                pass
        for squeak_hash in squeak_hashes:
            try:
                self.download_secret_key(squeak_hash)
            except Exception:
                pass

    def download_single_squeak(
            self,
            squeak_hash: bytes,
    ) -> None:
        self.download_squeak(squeak_hash)

    def download_single_squeak_secret_key(
            self,
            squeak_hash: bytes,
    ) -> None:
        self.download_secret_key(squeak_hash)

    def download_squeak(
            self,
            squeak_hash: bytes,
            min_block: Optional[int] = None,
            max_block: Optional[int] = None,
            pubkeys: Optional[List[SqueakPublicKey]] = None,
    ) -> None:
        # Download the squeak if not already owned.
        if self.squeak_store.get_squeak(squeak_hash):
            raise Exception('Squeak already saved.')

        # Download the squeak if not already owned.
        squeak = self.client.get_squeak(squeak_hash)

        # Check if the squeak is valid.
        if not squeak:
            raise Exception('Squeak not found.')
        if get_hash(squeak) != squeak_hash:
            raise Exception('Squeak has wrong hash.')
        if min_block and squeak.nBlockHeight < min_block:
            raise Exception('Squeak has block height below minimum.')
        if max_block and squeak.nBlockHeight > max_block:
            raise Exception('Squeak has block height above minimum.')
        if pubkeys and squeak.GetPubKey() not in pubkeys:
            raise Exception('Squeak has wronge pubkey.')

        # Save the squeak.
        self.squeak_store.save_squeak(squeak)

    def download_secret_key(self, squeak_hash: bytes) -> None:
        squeak = self.squeak_store.get_squeak(squeak_hash)

        # Check if squeak is already owned.
        if not squeak:
            raise Exception('Squeak is not already saved.')

        if self.squeak_store.get_squeak_secret_key(squeak_hash):
            raise Exception('Squeak secret key is already saved.')

        # Download the secret key if not already owned.
        secret_key = self.client.get_secret_key(squeak_hash)
        if secret_key:
            self.squeak_store.save_secret_key(squeak_hash, secret_key)
            return

        for received_offer in self.squeak_store.get_received_offers(squeak_hash):
            if received_offer.peer_address == self.peer.address:
                raise Exception('Received offer from this peer already saved.')

        # Download the offer if secret key not already owned.
        offer = self.client.get_offer(squeak_hash)
        if offer:
            self.squeak_store.handle_offer(
                squeak,
                offer,
                self.peer.address,
            )
