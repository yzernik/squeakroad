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
import threading
from typing import Optional

from squeaknode.core.exception import InvoiceSubscriptionError
from squeaknode.core.received_payment import ReceivedPayment
from squeaknode.core.sent_offer import SentOffer


logger = logging.getLogger(__name__)


class PaymentProcessor:

    def __init__(
        self,
        squeak_db,
        squeak_core,
        retry_s: int,
    ):
        self.squeak_db = squeak_db
        self.squeak_core = squeak_core
        self.retry_s = retry_s
        self.lock = threading.Lock()
        self.current_task = None

    def start_processing(self):
        with self.lock:
            if self.current_task is not None:
                self.current_task.stop_processing()
            self.current_task = PaymentProcessorTask(
                self.squeak_db,
                self.squeak_core,
                self.retry_s,
            )
            self.current_task.start_processing()

    def stop_processing(self):
        with self.lock:
            if self.current_task is not None:
                self.current_task.stop_processing()


class PaymentProcessorTask:

    def __init__(
        self,
        squeak_db,
        squeak_core,
        retry_s: int,
    ):
        self.squeak_db = squeak_db
        self.squeak_core = squeak_core
        self.retry_s = retry_s
        self.stopped = threading.Event()
        self.payments_result = None

    def start_processing(self):
        logger.info("Starting payment processor task.")
        threading.Thread(
            target=self.process_subscribed_invoices,
        ).start()

    def stop_processing(self):
        logger.info("Stopping payment processor task.")
        self.stopped.set()
        if self.payments_result is not None:
            self.payments_result.cancel_fn()

    def process_subscribed_invoices(self):
        while not self.stopped.is_set():
            try:
                # Use zero as starting settle index, so that no invoices are skipped.
                # latest_settle_index = self.get_latest_settle_index()
                latest_settle_index = 0
                logger.info("Starting payment subscription with settle index: {}".format(
                    latest_settle_index,
                ))
                self.payments_result = self.squeak_core.get_received_payments(
                    latest_settle_index,
                    self.get_sent_offer_for_payment_hash,
                )
                if self.stopped.is_set():
                    self.payments_result.cancel_fn()
                for received_payment in self.payments_result.result_stream:
                    self.handle_received_payment(received_payment)
            except InvoiceSubscriptionError:
                logger.error(
                    "Unable to subscribe invoices. Retrying in {} seconds...".format(
                        self.retry_s,
                    ),
                )
                self.stopped.wait(self.retry_s)

    def get_latest_settle_index(self) -> int:
        return self.squeak_db.get_latest_settle_index() or 0

    def get_sent_offer_for_payment_hash(self, payment_hash: bytes) -> Optional[SentOffer]:
        return self.squeak_db.get_sent_offer_by_payment_hash(
            payment_hash
        )

    def handle_received_payment(self, received_payment: ReceivedPayment):
        logger.info(
            "Got received payment: {}".format(received_payment))
        received_payment_id = self.squeak_db.insert_received_payment(
            received_payment,
        )
        if received_payment_id is not None:
            logger.debug(
                "Saved received payment: {}".format(received_payment))
        # # TODO: Should not be deleting sent offer.
        # self.squeak_db.delete_sent_offer(
        #     received_payment.payment_hash,
        # )
        self.squeak_db.set_sent_offer_paid(
            received_payment.payment_hash,
            paid=True,
        )
