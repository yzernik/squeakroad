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
from concurrent.futures import as_completed
from concurrent.futures import ThreadPoolExecutor
from concurrent.futures import wait
from typing import Optional

from squeaknode.client.peer_downloader import PeerDownloader
from squeaknode.core.squeak_peer import SqueakPeer
from squeaknode.node.squeak_store import SqueakStore

logger = logging.getLogger(__name__)


DOWNLOAD_TIMEOUT_S = 10


class NetworkController:

    def __init__(
            self,
            squeak_store: SqueakStore,
            proxy_host: Optional[str],
            proxy_port: Optional[int],
    ):
        self.squeak_store = squeak_store
        self.proxy_host = proxy_host
        self.proxy_port = proxy_port

    def get_downloader(self, peer: SqueakPeer):
        return PeerDownloader(
            peer,
            self.squeak_store,
            self.proxy_host,
            self.proxy_port,
        )

    def download_timeline(
            self,
            interest_block_interval: int,
    ) -> None:
        max_block = self.squeak_store.get_latest_block()
        min_block = max(0, max_block - interest_block_interval)
        followed_public_keys = self.squeak_store.get_followed_public_keys()
        peers = self.squeak_store.get_autoconnect_peers()
        downloaders = [
            self.get_downloader(peer)
            for peer in peers
        ]
        with ThreadPoolExecutor(50) as executor:
            # submit tasks and collect futures
            futures = [
                executor.submit(
                    downloader.download_interest_range,
                    min_block,
                    max_block,
                    followed_public_keys,
                )
                for downloader in downloaders
            ]
            # wait for at least one task to complete successfully.
            wait(futures)

    def download_single_squeak(self, squeak_hash: bytes) -> None:
        peers = self.squeak_store.get_autoconnect_peers()
        downloaders = [
            self.get_downloader(peer)
            for peer in peers
        ]
        executor = ThreadPoolExecutor(50)
        # submit tasks and collect futures
        futures = [
            executor.submit(
                downloader.download_single_squeak,
                squeak_hash
            )
            for downloader in downloaders]
        for future in as_completed(futures):
            err = future.exception()
            if err is None:
                break
        executor.shutdown(wait=False)

    def download_single_squeak_secret_key(self, squeak_hash: bytes) -> None:
        peers = self.squeak_store.get_autoconnect_peers()
        downloaders = [
            self.get_downloader(peer)
            for peer in peers
        ]
        with ThreadPoolExecutor(50) as executor:
            # submit tasks and collect futures
            futures = [
                executor.submit(
                    downloader.download_single_squeak_secret_key,
                    squeak_hash
                )
                for downloader in downloaders]
            # wait for all tasks to complete
            wait(futures)
