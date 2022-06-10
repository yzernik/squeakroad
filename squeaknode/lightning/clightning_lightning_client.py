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
import time
import uuid

from pyln.client import LightningRpc

from squeaknode.core.exception import InvoiceSubscriptionError
from squeaknode.lightning.info import Info
from squeaknode.lightning.invoice import Invoice
from squeaknode.lightning.invoice_stream import InvoiceStream
from squeaknode.lightning.lightning_client import LightningClient
from squeaknode.lightning.pay_req import PayReq
from squeaknode.lightning.payment import Payment


logger = logging.getLogger(__name__)


class CLightningClient(LightningClient):
    """Access a c-lightning instance using the Python API."""

    def __init__(
        self,
        rpc_path: str,
    ) -> None:
        self.rpc_path = rpc_path

    def init(self):
        # Create an instance of the LightningRpc object using the Core Lightning daemon on your computer.
        logger.info('initializing clightning client: {}'.format(self.rpc_path))
        self.lrpc = LightningRpc(self.rpc_path)

    def pay_invoice(self, payment_request: str) -> Payment:
        payment = self.lrpc.pay(payment_request)
        if payment['status'] == 'complete':
            return Payment(
                payment_preimage=bytes.fromhex(payment['payment_preimage']),
                payment_error='',
            )
        else:
            return Payment(
                payment_preimage=b'',
                payment_error='Payment failed.',
            )

    def get_info(self) -> Info:
        info = self.lrpc.getinfo()
        pubkey = info['id']
        binding = info.get('binding')
        uris = []
        if binding:
            for b in binding:
                address = b['address']
                if ':' not in address:  # TODO: Change type of uri to LightningAddress.
                    port = b['port']
                    uri = f"{pubkey}@{address}:{port}"
                    uris.append(uri)
        return Info(
            uris=uris,
        )

    def decode_pay_req(self, payment_request: str) -> PayReq:
        pay_req = self.lrpc.decodepay(payment_request)
        return PayReq(
            payment_hash=bytes.fromhex(pay_req['payment_hash']),
            payment_point=b'',  # TODO: Use real payment point.
            num_msat=pay_req['amount_msat'].millisatoshis,
            destination=pay_req['payee'],
            timestamp=int(pay_req['created_at']),
            expiry=int(pay_req['expiry']),
        )

    def create_invoice(self, preimage: bytes, amount_msat: int) -> Invoice:
        created_invoice = self.lrpc.invoice(
            amount_msat,
            label=str(uuid.uuid4()),
            description="Squeaknode invoice",
            preimage=preimage.hex(),
        )
        creation_time = int(time.time())
        expiry = int(created_invoice['expires_at']) - creation_time
        return Invoice(
            r_hash=bytes.fromhex(created_invoice['payment_hash']),
            payment_request=created_invoice['bolt11'],
            value_msat=amount_msat,
            settled=False,
            settle_index=0,
            creation_date=creation_time,
            expiry=expiry,
        )

    def subscribe_invoices(self, settle_index: int) -> InvoiceStream:

        def cancel_fn():
            return None

        def get_invoice_stream():
            try:
                pay_index = settle_index
                while True:
                    payment = self.lrpc.waitanyinvoice(lastpay_index=pay_index)
                    if payment['status'] == 'paid':
                        pay_index = payment.get('pay_index') or 0
                        yield Invoice(
                            r_hash=bytes.fromhex(payment['payment_hash']),
                            payment_request=payment.get('bolt11') or '',
                            value_msat=payment.get('value_msat'),
                            settled=True,
                            settle_index=pay_index,
                            creation_date=0,  # This value is ignored.
                            expiry=payment['expires_at'],
                        )
            except Exception:
                raise InvoiceSubscriptionError()

        return InvoiceStream(
            cancel=cancel_fn,
            result_stream=get_invoice_stream(),
        )
