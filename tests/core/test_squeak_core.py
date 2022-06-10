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
import pytest

from squeaknode.bitcoin.bitcoin_client import BitcoinClient
from squeaknode.bitcoin.block_info import BlockInfo
from squeaknode.core.squeak_core import SqueakCore
from squeaknode.core.squeaks import make_squeak_with_block
from squeaknode.lightning.info import Info
from squeaknode.lightning.invoice import Invoice
from squeaknode.lightning.invoice_stream import InvoiceStream
from squeaknode.lightning.lightning_client import LightningClient
from squeaknode.lightning.pay_req import PayReq
from squeaknode.lightning.payment import Payment
from tests.utils import gen_random_hash


@pytest.fixture
def info(uris):
    yield Info(
        uris=uris,
    )


@pytest.fixture
def info_with_no_uris():
    yield Info(
        uris=[],
    )


@pytest.fixture
def invoice(payment_hash, payment_request, price_msat, creation_date, expiry):
    yield Invoice(
        r_hash=payment_hash,
        payment_request=payment_request,
        value_msat=price_msat,
        settled=False,
        settle_index=0,
        creation_date=creation_date,
        expiry=expiry,
    )


@pytest.fixture
def settled_invoice(invoice, settle_index):
    yield invoice._replace(
        settled=True,
        settle_index=settle_index,
    )


@pytest.fixture
def unsettled_invoice(invoice, settle_index):
    yield invoice


@pytest.fixture
def pay_req(
        payment_hash,
        payment_point,
        price_msat,
        payment_request,
        seller_pubkey,
        creation_date,
        expiry,
):
    yield PayReq(
        payment_hash=payment_hash,
        payment_point=payment_point,
        num_msat=price_msat,
        destination=seller_pubkey,
        timestamp=creation_date,
        expiry=expiry,
    )


@pytest.fixture
def pay_req_with_no_payment_point(
        payment_hash,
        price_msat,
        payment_request,
        seller_pubkey,
        creation_date,
        expiry,
):
    yield PayReq(
        payment_hash=payment_hash,
        payment_point=b'',
        num_msat=price_msat,
        destination=seller_pubkey,
        timestamp=creation_date,
        expiry=expiry,
    )


@pytest.fixture
def successful_payment(preimage):
    yield Payment(
        payment_preimage=preimage,
        payment_error='',
    )


@pytest.fixture
def failed_payment(payment_request):
    yield Payment(
        payment_preimage=b'',
        payment_error='Payment failed.',
    )


@pytest.fixture
def invoice_stream(settled_invoice, unsettled_invoice):
    # Length of invoices list is 1.
    invoices = [
        settled_invoice,
        # unsettled_invoice,
    ]
    yield InvoiceStream(
        cancel=lambda: None,
        result_stream=iter(invoices),
    )


@pytest.fixture
def other_block_info():
    yield BlockInfo(
        block_height=5678,
        block_hash=gen_random_hash(),
        block_header=b'',
    )


@pytest.fixture
def other_squeak(private_key, squeak_content, other_block_info):
    squeak, _ = make_squeak_with_block(
        private_key,
        squeak_content,
        other_block_info.block_height,
        other_block_info.block_hash,
    )
    yield squeak


class MockBitcoinClient(BitcoinClient):

    def __init__(self, best_block_info):
        self.best_block_info = best_block_info

    def get_best_block_info(self) -> BlockInfo:
        return self.best_block_info

    def get_block_info_by_height(self, block_height: int) -> BlockInfo:
        return self.best_block_info

    def get_block_hash(self, block_height: int) -> bytes:
        return self.best_block_info.block_hash

    def get_block_header(self, block_hash: bytes, verbose: bool) -> bytes:
        return self.best_block_info.block_header


class MockLightningClient(LightningClient):

    def __init__(self, info, invoice, pay_req, payment, invoice_stream):
        self.info = info
        self.invoice = invoice
        self.pay_req = pay_req
        self.payment = payment
        self.invoice_stream = invoice_stream

    def init(self):
        pass

    def get_info(self):
        return self.info

    def create_invoice(self, preimage: bytes, amount_msat: int):
        return self.invoice

    def decode_pay_req(self, payment_request: str):
        return self.pay_req

    def pay_invoice(self, payment_request: str):
        return self.payment

    def subscribe_invoices(self, settle_index: int):
        return self.invoice_stream


@pytest.fixture
def bitcoin_client(block_info):
    return MockBitcoinClient(block_info)


@pytest.fixture
def lightning_client(info, invoice, pay_req, successful_payment, invoice_stream):
    return MockLightningClient(info, invoice, pay_req, successful_payment, invoice_stream)


@pytest.fixture
def lightning_client_with_failed_payment(info, invoice, pay_req, failed_payment, invoice_stream):
    return MockLightningClient(info, invoice, pay_req, failed_payment, invoice_stream)


@pytest.fixture
def lightning_client_with_no_uris(info_with_no_uris, invoice, pay_req, failed_payment, invoice_stream):
    return MockLightningClient(info_with_no_uris, invoice, pay_req, failed_payment, invoice_stream)


@pytest.fixture
def lightning_client_with_no_payment_point(info, invoice, pay_req_with_no_payment_point, successful_payment, invoice_stream):
    return MockLightningClient(info, invoice, pay_req_with_no_payment_point, successful_payment, invoice_stream)


@pytest.fixture
def squeak_core(bitcoin_client, lightning_client):
    yield SqueakCore(bitcoin_client, lightning_client)


@pytest.fixture
def squeak_core_with_failed_payment(bitcoin_client, lightning_client_with_failed_payment):
    yield SqueakCore(bitcoin_client, lightning_client_with_failed_payment)


@pytest.fixture
def squeak_core_with_no_uris(bitcoin_client, lightning_client_with_no_uris):
    yield SqueakCore(bitcoin_client, lightning_client_with_no_uris)


@pytest.fixture
def squeak_core_with_no_payment_point(bitcoin_client, lightning_client_with_no_payment_point):
    yield SqueakCore(bitcoin_client, lightning_client_with_no_payment_point)


@pytest.fixture
def created_offer(squeak_core, squeak, secret_key, peer_address, price_msat, nonce):
    yield squeak_core.create_offer(
        squeak,
        secret_key,
        peer_address,
        price_msat,
        nonce,
    )


@pytest.fixture
def packaged_offer(squeak_core, created_offer):
    yield squeak_core.package_offer(created_offer, None)


@pytest.fixture
def unpacked_offer(squeak_core, squeak, packaged_offer, peer_address):
    yield squeak_core.unpack_offer(
        squeak,
        packaged_offer,
        peer_address,
        check_payment_point=True,
    )


@pytest.fixture
def completed_payment(squeak_core, unpacked_offer):
    yield squeak_core.pay_offer(unpacked_offer)


@pytest.fixture
def received_payments_stream(squeak_core, unpacked_offer):
    yield squeak_core.pay_offer(unpacked_offer)


def test_make_squeak(
        squeak_core,
        signing_profile,
        squeak_content,
        block_header
):
    created_squeak, created_secret_key = squeak_core.make_squeak(
        signing_profile,
        squeak_content,
    )
    decrypted_created_content = squeak_core.get_decrypted_content(
        created_squeak,
        created_secret_key,
    )

    assert decrypted_created_content == squeak_content


def test_make_squeak_with_contact_profile(
        squeak_core,
        contact_profile,
        squeak_content,
):
    with pytest.raises(Exception) as excinfo:
        created_squeak, created_secret_key = squeak_core.make_squeak(
            contact_profile,
            squeak_content,
        )
    assert "Can't make squeak with a contact profile." in str(excinfo.value)


def test_make_private_squeak(
        squeak_core,
        signing_profile,
        squeak_content,
        block_header,
        recipient_signing_profile,
        recipient_contact_profile,
):
    created_squeak, created_secret_key = squeak_core.make_squeak(
        signing_profile,
        squeak_content,
        recipient_profile=recipient_contact_profile,
    )
    recipient_decrypted_content = squeak_core.get_decrypted_content(
        created_squeak,
        created_secret_key,
        recipient_profile=recipient_signing_profile,
    )
    author_decrypted_content = squeak_core.get_decrypted_content(
        created_squeak,
        created_secret_key,
        author_profile=signing_profile,
    )

    assert recipient_decrypted_content == squeak_content
    assert author_decrypted_content == squeak_content


def test_make_resqueak(
        squeak_core,
        signing_profile,
        squeak_hash,
        block_header
):
    created_resqueak = squeak_core.make_resqueak(
        signing_profile,
        squeak_hash,
    )

    assert created_resqueak.hashResqueakSqk == squeak_hash
    squeak_core.check_squeak(created_resqueak)


def test_get_block_header(
        squeak_core,
        squeak,
        block_info,
):
    block_header = squeak_core.get_block_header(squeak)

    assert block_header == block_info.block_header


def test_get_block_header_invalid_block_hash(
        squeak_core,
        other_squeak,
):
    with pytest.raises(Exception) as excinfo:
        squeak_core.get_block_header(other_squeak)
    assert "Block hash incorrect." in str(excinfo.value)


def test_check_squeak(squeak_core, squeak):
    squeak_core.check_squeak(squeak)


def test_get_best_block_height(squeak_core, block_info):
    best_block_height = squeak_core.get_best_block_height()

    assert best_block_height == block_info.block_height


def test_create_offer(
        squeak_core,
        squeak,
        secret_key,
        peer_address,
        price_msat,
        nonce,
        invoice,
        sent_offer,
):
    created_offer = squeak_core.create_offer(
        squeak,
        secret_key,
        peer_address,
        price_msat,
        nonce,
    )

    assert created_offer == sent_offer


def test_packaged_offer(squeak, packaged_offer, offer):

    assert packaged_offer == offer


def test_package_offer_with_no_lnd_uris(
        squeak_core_with_no_uris,
        created_offer,
):
    packaged_offer = squeak_core_with_no_uris.package_offer(
        created_offer,
        None,
    )

    assert packaged_offer.host == ''
    assert packaged_offer.port == 0


def test_package_offer_with_no_lnd_uris_with_external_address(
        squeak_core_with_no_uris,
        created_offer,
        external_lightning_address,
):
    packaged_offer = squeak_core_with_no_uris.package_offer(
        created_offer,
        external_lightning_address,
    )

    assert packaged_offer.host == external_lightning_address.host
    assert packaged_offer.port == external_lightning_address.port


def test_unpacked_offer(unpacked_offer, received_offer):

    assert unpacked_offer == received_offer


def test_unpack_offer_invalid_squeak_hash(squeak_core, other_squeak, packaged_offer, peer_address):
    with pytest.raises(Exception) as excinfo:
        squeak_core.unpack_offer(
            other_squeak,
            packaged_offer,
            peer_address,
        )
    assert "does not match squeak hash" in str(excinfo.value)


def test_unpacked_offer_bad_payment_point(squeak_core_with_no_payment_point, squeak, packaged_offer, peer_address):
    with pytest.raises(Exception) as excinfo:
        squeak_core_with_no_payment_point.unpack_offer(
            squeak,
            packaged_offer,
            peer_address,
            check_payment_point=True,
        )
    assert "Invalid payment point." in str(excinfo.value)


def test_unpacked_offer_bad_payment_point_skip_check(
        squeak_core_with_no_payment_point,
        squeak,
        packaged_offer,
        peer_address,
        received_offer,
):
    unpacked_offer = squeak_core_with_no_payment_point.unpack_offer(
        squeak,
        packaged_offer,
        peer_address,
    )

    assert unpacked_offer == received_offer


def test_completed_payment(
        completed_payment,
        squeak,
        price_msat,
        secret_key,
        seller_pubkey,
        sent_payment,
):

    assert completed_payment == sent_payment


def test_send_payment_with_failure(squeak_core_with_failed_payment, unpacked_offer):

    with pytest.raises(Exception) as excinfo:
        squeak_core_with_failed_payment.pay_offer(unpacked_offer)
    assert "Payment failed with error" in str(excinfo.value)


def test_unlock_squeak(squeak_core, squeak, squeak_content, completed_payment):
    decrypted_content = squeak_core.get_decrypted_content(
        squeak,
        completed_payment.secret_key,
    )

    assert decrypted_content == squeak_content


def test_get_received_payments(squeak_core, settle_index, sent_offer, received_payment):
    def get_sent_offer_fn(payment_hash):
        return sent_offer

    received_payments_stream = squeak_core.get_received_payments(
        settle_index,
        get_sent_offer_fn,
    )
    received_payments = list(received_payments_stream.result_stream)

    assert len(received_payments) == 1
    assert received_payments[0] == received_payment
