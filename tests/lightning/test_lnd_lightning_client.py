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
import mock
import pytest

from proto import lnd_pb2
from squeaknode.lightning.info import Info
from squeaknode.lightning.invoice import Invoice
from squeaknode.lightning.lnd_lightning_client import LNDLightningClient
from squeaknode.lightning.pay_req import PayReq
from squeaknode.lightning.payment import Payment
from tests.utils import gen_random_hash


@pytest.fixture
def lnd_host():
    yield "fake_lnd_host"


@pytest.fixture
def lnd_port():
    yield 9876


@pytest.fixture
def tls_cert_path():
    yield "fake_tls_cert_path"


@pytest.fixture
def macaroon_path():
    yield "fake_macaroon_path"


@pytest.fixture
def preimage():
    yield gen_random_hash()


@pytest.fixture
def payment_hash(preimage):
    # TODO: This should be the hash of the preimage
    yield gen_random_hash()


@pytest.fixture
def payment_hash_str(payment_hash):
    yield payment_hash.hex()


@pytest.fixture
def payment_point(preimage):
    yield b''


@pytest.fixture
def price_msat():
    yield 33333


@pytest.fixture
def creation_date():
    yield 777777


@pytest.fixture
def expiry():
    yield 5555


@pytest.fixture
def payment_request():
    yield "fake_payment_request"


@pytest.fixture
def destination():
    yield "fake_payment_request"


@pytest.fixture
def timestamp():
    yield 8888888


@pytest.fixture
def rpc_invoice(preimage, payment_hash, payment_request, price_msat, creation_date, expiry):
    yield lnd_pb2.Invoice(
        r_preimage=preimage,
        r_hash=payment_hash,
        payment_request=payment_request,
        value_msat=price_msat,
        settled=True,
        settle_index=0,
        creation_date=creation_date,
        expiry=expiry,
    )


@pytest.fixture
def invoice(payment_hash, payment_request, price_msat, creation_date, expiry):
    yield Invoice(
        r_hash=payment_hash,
        payment_request=payment_request,
        value_msat=price_msat,
        settled=True,
        settle_index=0,
        creation_date=creation_date,
        expiry=expiry,
    )


@pytest.fixture
def add_invoice_response(payment_hash, payment_request):
    yield lnd_pb2.AddInvoiceResponse(
        r_hash=payment_hash,
        payment_request=payment_request,
    )


@pytest.fixture
def uris():
    yield [
        'foobar.com:12345'
        'fakehost.com:56789'
    ]


@pytest.fixture
def get_info_response(uris):
    yield lnd_pb2.GetInfoResponse(
        uris=uris,
    )


@pytest.fixture
def info(uris):
    yield Info(
        uris=uris,
    )


@pytest.fixture
def send_request(payment_request):
    yield lnd_pb2.SendRequest(
        payment_request=payment_request,
    )


@pytest.fixture
def send_response(preimage, payment_hash):
    yield lnd_pb2.SendResponse(
        payment_preimage=preimage,
        payment_hash=payment_hash,
    )


@pytest.fixture
def payment(preimage):
    yield Payment(
        payment_preimage=preimage,
        payment_error='',
    )


@pytest.fixture
def decode_pay_req_request(payment_request):
    yield lnd_pb2.PayReqString(
        pay_req=payment_request,
    )


@pytest.fixture
def decode_pay_req_response(
        payment_hash_str,
        price_msat,
        payment_request,
        destination,
        timestamp,
        expiry,
):
    yield lnd_pb2.PayReq(
        payment_hash=payment_hash_str,
        num_msat=price_msat,
        destination=destination,
        timestamp=timestamp,
        expiry=expiry,
    )


@pytest.fixture
def pay_req(
        payment_hash,
        payment_point,
        price_msat,
        payment_request,
        destination,
        timestamp,
        expiry,
):
    yield PayReq(
        payment_hash=payment_hash,
        payment_point=payment_point,
        num_msat=price_msat,
        destination=destination,
        timestamp=timestamp,
        expiry=expiry,
    )


@pytest.fixture
def lookup_invoice_request(payment_hash_str):
    yield lnd_pb2.PaymentHash(
        r_hash_str=payment_hash_str,
    )


@pytest.fixture
def make_lightning_client(lnd_host, lnd_port, tls_cert_path, macaroon_path):
    client = LNDLightningClient(
        host=lnd_host,
        port=lnd_port,
        tls_cert_path=tls_cert_path,
        macaroon_path=macaroon_path,
    )
    with mock.patch.object(client, '_get_stub', autospec=True) as mock_get_stub:
        def fn(stub):
            mock_get_stub.return_value = stub
            client.init()
            return client
        yield fn


def test_add_invoice(make_lightning_client, preimage, price_msat, rpc_invoice, invoice, add_invoice_response):
    mock_stub = mock.MagicMock()
    mock_stub.AddInvoice.return_value = add_invoice_response
    client = make_lightning_client(mock_stub)
    response = client.add_invoice(
        preimage, price_msat)
    (call_invoice,) = mock_stub.AddInvoice.call_args.args

    assert type(call_invoice) is lnd_pb2.Invoice
    assert call_invoice.r_preimage == preimage
    assert call_invoice.value_msat == price_msat
    assert type(response) is lnd_pb2.AddInvoiceResponse
    assert response == add_invoice_response


def test_get_info(make_lightning_client, get_info_response, info):
    mock_stub = mock.MagicMock()
    mock_stub.GetInfo.return_value = get_info_response
    client = make_lightning_client(mock_stub)
    response = client.get_info()
    (call_get_info,) = mock_stub.GetInfo.call_args.args

    assert type(call_get_info) is lnd_pb2.GetInfoRequest
    assert response == info


def test_pay_invoice(make_lightning_client, payment_request, send_request, send_response, payment):
    mock_stub = mock.MagicMock()
    mock_stub.SendPaymentSync.return_value = send_response
    client = make_lightning_client(mock_stub)
    response = client.pay_invoice(payment_request)
    (call_send_request,) = mock_stub.SendPaymentSync.call_args.args

    assert type(call_send_request) is lnd_pb2.SendRequest
    assert call_send_request.payment_request == payment_request
    assert response == payment


def test_decode_pay_req(make_lightning_client, payment_request, decode_pay_req_request, decode_pay_req_response, pay_req):
    mock_stub = mock.MagicMock()
    mock_stub.DecodePayReq.return_value = decode_pay_req_response
    client = make_lightning_client(mock_stub)
    response = client.decode_pay_req(payment_request)
    (call_decode_pay_req_request,) = mock_stub.DecodePayReq.call_args.args

    assert type(call_decode_pay_req_request) is lnd_pb2.PayReqString
    assert call_decode_pay_req_request.pay_req == payment_request
    assert response == pay_req


def test_lookup_invoice(make_lightning_client, payment_hash_str, lookup_invoice_request, rpc_invoice):
    mock_stub = mock.MagicMock()
    mock_stub.LookupInvoice.return_value = rpc_invoice
    client = make_lightning_client(mock_stub)
    response = client.lookup_invoice(payment_hash_str)
    (call_lookup_invoice_request,) = mock_stub.LookupInvoice.call_args.args

    assert type(call_lookup_invoice_request) is lnd_pb2.PaymentHash
    assert call_lookup_invoice_request.r_hash_str == payment_hash_str
    assert response == rpc_invoice


def test_create_invoice(make_lightning_client, preimage, price_msat, payment_hash_str, add_invoice_response, rpc_invoice, invoice):
    mock_stub = mock.MagicMock()
    client = make_lightning_client(mock_stub)
    with mock.patch.object(client, 'add_invoice', autospec=True) as mock_add_invoice, \
            mock.patch.object(client, 'lookup_invoice', autospec=True) as mock_lookup_invoice:
        mock_add_invoice.return_value = add_invoice_response
        mock_lookup_invoice.return_value = rpc_invoice
        response = client.create_invoice(preimage, price_msat)
        (call_preimage, call_price_msat,) = mock_add_invoice.call_args.args
        (call_payment_hash_str,) = mock_lookup_invoice.call_args.args

        assert call_preimage == preimage
        assert call_price_msat == price_msat
        assert call_payment_hash_str == payment_hash_str
        assert response == invoice


def test_subscribe_invoices(make_lightning_client, settle_index, invoice):
    mock_subscribe_invoices_result = mock.MagicMock()
    invoices = [invoice, invoice, invoice]
    mock_subscribe_invoices_result.__iter__.return_value = invoices
    mock_stub = mock.MagicMock()
    mock_stub.SubscribeInvoices.return_value = mock_subscribe_invoices_result
    client = make_lightning_client(mock_stub)
    response = client.subscribe_invoices(settle_index)
    (call_subscribe_invoice_subscription,
     ) = mock_stub.SubscribeInvoices.call_args.args

    assert call_subscribe_invoice_subscription.settle_index == settle_index
    assert list(response.result_stream) == invoices

    response.cancel()

    assert mock_subscribe_invoices_result.cancel.called_once_with()
