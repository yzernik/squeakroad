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
import queue
import threading
from contextlib import contextmanager

from squeaknode.node.squeak_store import SqueakStore

logger = logging.getLogger(__name__)

DEFAULT_MAX_QUEUE_SIZE = 1000
DEFAULT_UPDATE_INTERVAL_S = 1


class ReceivedPaymentsSubscriptionClient:
    """Class that can be used to get a subscription to a stream
    of received payments.

    `initial_index` parameter refers to the `received_payment_id` that will be
    the first in the result stream.
    """

    def __init__(
        self,
        # squeak_db,
        squeak_store: SqueakStore,
        initial_index: int,
        stopped: threading.Event,
        max_queue_size=DEFAULT_MAX_QUEUE_SIZE,
        update_interval_s=DEFAULT_UPDATE_INTERVAL_S,
    ):
        self.squeak_store = squeak_store
        self.initial_index = initial_index
        self.stopped = stopped
        self.update_interval_s = update_interval_s
        self.q: queue.Queue = queue.Queue(max_queue_size)

    @contextmanager
    def open_subscription(self):
        # Start the thread to populate the queue.
        threading.Thread(
            target=self.populate_queue,
        ).start()
        try:
            logger.info("Before yielding received payment client...")
            yield self
            logger.info("After yielding received payment client...")
        finally:
            logger.info("Stopping received payment client...")
            self.stopped.set()
            logger.info("Stopped received payment client...")

    def populate_queue(self):
        payment_index = self.initial_index
        while not self.stopped.is_set():
            try:
                for payment in self.get_latest_received_payments(payment_index):
                    self.q.put(payment)
                    payment_index = payment.received_payment_id
                    logger.info(
                        "Added payment to queue. Size: {}".format(
                            self.q.qsize())
                    )
            except Exception:
                logger.error(
                    "Exception while populating queue.",
                    exc_info=True,
                )
            self.stopped.wait(self.update_interval_s)
        # Put the poison pill
        logger.info("Putting poison pill in queue...")
        self.q.put(None)

    def get_latest_received_payments(self, payment_index):
        # return self.squeak_db.yield_received_payments_from_index(
        #     payment_index,
        # )
        return self.squeak_store.yield_received_payments_from_index(
            payment_index,
        )

    def get_received_payments(self):
        while True:
            payment = self.q.get()
            if payment is None:
                logger.debug("Poison pill swallowed.")
                return
            yield payment
            self.q.task_done()
            logger.info(
                "Removed payment from queue. Size: {}".format(
                    self.q.qsize())
            )
