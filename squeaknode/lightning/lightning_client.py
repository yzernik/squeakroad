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
from abc import abstractmethod

from squeaknode.lightning.info import Info
from squeaknode.lightning.invoice import Invoice
from squeaknode.lightning.invoice_stream import InvoiceStream
from squeaknode.lightning.pay_req import PayReq
from squeaknode.lightning.payment import Payment

logger = logging.getLogger(__name__)


class LightningClient(ABC):

    @abstractmethod
    def init(self):
        """Initialize the lightning client.
        Returns:
            None
        """

    @abstractmethod
    def get_info(self) -> Info:
        """Get info about the lightning node.

        Returns:
            Info: an object containing information about the node.

        Raises:
            LightningRequestError: If the request fails.
        """

    @abstractmethod
    def create_invoice(self, preimage: bytes, amount_msat: int) -> Invoice:
        """Add a new invoice.

        Args:
            preimage: The lightning invoice preimage.
            amount_msat: The amount of the invoice in msats.

        Returns:
            Invoice: an object representing a lightning invoice.

        Raises:
            LightningRequestError: If the request fails.
        """

    @abstractmethod
    def decode_pay_req(self, payment_request: str) -> PayReq:
        """Get the decoded payment request.

        Args:
            payment_request: The payment request as a string.

        Returns:
            PayReq: an object representing the payment request.

        Raises:
            LightningRequestError: If the request fails.
        """

    @abstractmethod
    def pay_invoice(self, payment_request: str) -> Payment:
        """Pay an invoice with a given payment_request.

        Args:
            payment_request: The payment request as a string.

        Returns:
            Payment: The payment result.

        args:
        payment_request -- the payment_request as a string
        """

    @abstractmethod
    def subscribe_invoices(self, settle_index: int) -> InvoiceStream:
        """Get a stream of settled invoices for received payments.
        # TODO: use map function to convert type of items in stream.

        Args:
            settle_index: The settle index from which to start streaming.

        Returns:
            InvoiceStream: an object containing the stream of invoices.

        Raises:
            LightningRequestError: If the request fails.
        """
