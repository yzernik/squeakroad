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

from squeaknode.client.network_controller import NetworkController
from squeaknode.node.periodic_worker import PeriodicWorker
from squeaknode.node.squeak_store import SqueakStore

logger = logging.getLogger(__name__)


class SqueakDownloadWorker(PeriodicWorker):
    def __init__(
        self,
        squeak_store: SqueakStore,
        network_controller: NetworkController,
        download_timeline_interval_s: int,
        interest_block_interval: int,
    ):
        self.squeak_store = squeak_store
        self.download_timeline_interval_s = download_timeline_interval_s
        self.interest_block_interval = interest_block_interval
        self.network_controller = network_controller

    def work_fn(self):
        self.network_controller.download_timeline(
            self.interest_block_interval,
        )

    def get_interval_s(self):
        return self.download_timeline_interval_s

    def get_name(self):
        return "squeak_download_worker"
