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
import codecs
import logging
import os

import grpc

from proto import lnd_pb2
from proto import lnd_pb2_grpc
from squeaknode.core.exception import InvoiceSubscriptionError
from squeaknode.lightning.info import Info
from squeaknode.lightning.invoice import Invoice
from squeaknode.lightning.invoice_stream import InvoiceStream
from squeaknode.lightning.lightning_client import LightningClient
from squeaknode.lightning.pay_req import PayReq
from squeaknode.lightning.payment import Payment

logger = logging.getLogger(__name__)


# Due to updated ECDSA generated tls.cert we need to let gprc know that
# we need to use that cipher suite otherwise there will be a handhsake
# error when we communicate with the lnd rpc server.
os.environ["GRPC_SSL_CIPHER_SUITES"] = "HIGH+ECDSA"
# os.environ["GRPC_VERBOSITY"] = "DEBUG"


class LNDLightningClient(LightningClient):
    """Access a lightning deamon using RPC."""

    def __init__(
        self,
        host: str,
        port: int,
        tls_cert_path: str,
        macaroon_path: str,
    ) -> None:
        self.host = host
        self.port = port
        self.tls_cert_path = tls_cert_path
        self.macaroon_path = macaroon_path
        # self.stub = None

    def init(self):
        self.stub = self._get_stub()

    def _get_stub(self):  # pragma: no cover
        url = "{}:{}".format(self.host, self.port)

        # Lnd cert is at ~/.lnd/tls.cert on Linux and
        # ~/Library/Application Support/Lnd/tls.cert on Mac
        cert = open(os.path.expanduser(self.tls_cert_path), "rb").read()
        cert_creds = grpc.ssl_channel_credentials(cert)

        # Lnd admin macaroon is at ~/.lnd/data/chain/bitcoin/simnet/admin.macaroon on Linux and
        # ~/Library/Application Support/Lnd/data/chain/bitcoin/simnet/admin.macaroon on Mac
        with open(os.path.expanduser(self.macaroon_path), "rb") as f:
            macaroon_bytes = f.read()
            macaroon = codecs.encode(macaroon_bytes, "hex")
            self.macaroon = codecs.encode(macaroon_bytes, "hex")

        def metadata_callback(context, callback):
            # for more info see grpc docs
            callback([("macaroon", macaroon)], None)

        # now build meta data credentials
        auth_creds = grpc.metadata_call_credentials(metadata_callback)

        # combine the cert credentials and the macaroon auth credentials
        # such that every call is properly encrypted and authenticated
        combined_creds = grpc.composite_channel_credentials(
            cert_creds, auth_creds)

        # finally pass in the combined credentials when creating a channel
        channel = grpc.secure_channel(url, combined_creds)
        return lnd_pb2_grpc.LightningStub(channel)

    def add_invoice(self, preimage: bytes, amount_msat: int) -> lnd_pb2.AddInvoiceResponse:
        invoice = lnd_pb2.Invoice(
            r_preimage=preimage,
            value_msat=amount_msat,
        )
        return self.stub.AddInvoice(invoice)

    def pay_invoice(self, payment_request: str) -> Payment:
        send_payment_request = lnd_pb2.SendRequest(
            payment_request=payment_request,
        )
        send_payment_response = self.stub.SendPaymentSync(send_payment_request)
        return Payment(
            payment_preimage=send_payment_response.payment_preimage,
            payment_error=send_payment_response.payment_error,
        )

    def get_info(self) -> Info:
        get_info_request = lnd_pb2.GetInfoRequest()
        get_info_response = self.stub.GetInfo(
            get_info_request,
        )
        return Info(
            uris=get_info_response.uris,
        )

    def decode_pay_req(self, payment_request: str) -> PayReq:
        decode_pay_req_request = lnd_pb2.PayReqString(
            pay_req=payment_request,
        )
        decode_pay_req_response = self.stub.DecodePayReq(
            decode_pay_req_request,
        )
        return PayReq(
            payment_hash=bytes.fromhex(decode_pay_req_response.payment_hash),
            payment_point=b'',  # TODO: Use real payment point.
            num_msat=decode_pay_req_response.num_msat,
            destination=decode_pay_req_response.destination,
            timestamp=decode_pay_req_response.timestamp,
            expiry=decode_pay_req_response.expiry,
        )

    def lookup_invoice(self, r_hash_str: str) -> lnd_pb2.Invoice:
        payment_hash = lnd_pb2.PaymentHash(
            r_hash_str=r_hash_str,
        )
        return self.stub.LookupInvoice(payment_hash)

    def create_invoice(self, preimage: bytes, amount_msat: int) -> Invoice:
        add_invoice_response = self.add_invoice(preimage, amount_msat)
        payment_hash = add_invoice_response.r_hash
        lookup_invoice_response = self.lookup_invoice(
            payment_hash.hex()
        )
        return Invoice(
            r_hash=lookup_invoice_response.r_hash,
            payment_request=lookup_invoice_response.payment_request,
            value_msat=amount_msat,
            settled=lookup_invoice_response.settled,
            settle_index=lookup_invoice_response.settle_index,
            creation_date=lookup_invoice_response.creation_date,
            expiry=lookup_invoice_response.expiry,
        )

    def subscribe_invoices(self, settle_index: int) -> InvoiceStream:
        subscribe_invoices_request = lnd_pb2.InvoiceSubscription(
            settle_index=settle_index,
        )
        subscribe_result = self.stub.SubscribeInvoices(
            subscribe_invoices_request,
        )

        def get_invoice_stream():
            try:
                for invoice in subscribe_result:
                    if invoice.settled:
                        yield Invoice(
                            r_hash=invoice.r_hash,
                            payment_request=invoice.payment_request,
                            value_msat=invoice.value_msat,
                            settled=invoice.settled,
                            settle_index=invoice.settle_index,
                            creation_date=invoice.creation_date,
                            expiry=invoice.expiry,
                        )
            except grpc.RpcError as e:
                if e.code() != grpc.StatusCode.CANCELLED:
                    raise InvoiceSubscriptionError()

        return InvoiceStream(
            cancel=subscribe_result.cancel,
            result_stream=get_invoice_stream(),
        )
