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
from bitcoin.core import CBlockHeader
from squeak.core.elliptic import payment_point_bytes_from_scalar_bytes
from squeak.core.keys import SqueakPrivateKey

from squeaknode.bitcoin.block_info import BlockInfo
from squeaknode.core.download_result import DownloadResult
from squeaknode.core.lightning_address import LightningAddressHostPort
from squeaknode.core.offer import Offer
from squeaknode.core.payment_summary import PaymentSummary
from squeaknode.core.peer_address import Network
from squeaknode.core.peer_address import PeerAddress
from squeaknode.core.received_offer import ReceivedOffer
from squeaknode.core.received_payment import ReceivedPayment
from squeaknode.core.received_payment_summary import ReceivedPaymentSummary
from squeaknode.core.secret_keys import add_tweak
from squeaknode.core.secret_keys import generate_tweak
from squeaknode.core.sent_offer import SentOffer
from squeaknode.core.sent_payment import SentPayment
from squeaknode.core.sent_payment_summary import SentPaymentSummary
from squeaknode.core.squeak_entry import SqueakEntry
from squeaknode.core.squeak_peer import SqueakPeer
from squeaknode.core.squeaks import get_hash
from squeaknode.core.squeaks import make_resqueak_with_block
from squeaknode.core.squeaks import make_squeak_with_block
from squeaknode.core.user_config import UserConfig
from tests.utils import gen_contact_profile
from tests.utils import gen_signing_profile
from tests.utils import sha256


@pytest.fixture
def private_key():
    yield SqueakPrivateKey.generate()


@pytest.fixture
def private_key_bytes(private_key):
    yield private_key.to_bytes()


@pytest.fixture
def public_key(private_key):
    yield private_key.get_public_key()


@pytest.fixture
def recipient_private_key():
    yield SqueakPrivateKey.generate()


@pytest.fixture
def recipient_public_key(recipient_private_key):
    yield recipient_private_key.get_public_key()


@pytest.fixture
def block_count():
    yield 555


@pytest.fixture
def block_hash_str():
    # Block hash of block at height 555
    yield '00000000edade40797e3c4bf27edeb65733d1884beaa8c502a89d50a54111e1c'


@pytest.fixture
def block_hash(block_hash_str):
    yield bytes.fromhex(block_hash_str)


@pytest.fixture
def block_header_str():
    # Block header of block at height 555
    yield '0100000079c30d2c23727a1e9f5feda4e7feb8ea0bda2ab98e23e7f6a9cf594f00000000b0de897e42fa7a3b5c3a6bfb8e797acf4ffbc16169394b03ad93296524ed633dcfef6e49ffff001d36d19a6c'


@pytest.fixture
def block_header_bytes(block_header_str):
    yield bytes.fromhex(block_header_str)


@pytest.fixture
def block_header(block_header_bytes):
    yield CBlockHeader.deserialize(block_header_bytes)


@pytest.fixture
def block_time(block_header):
    yield block_header.nTime


@pytest.fixture
def block_info(block_count, block_hash, block_header):
    yield BlockInfo(
        block_height=block_count,
        block_hash=block_hash,
        block_header=block_header,
    )


@pytest.fixture
def squeak_content():
    yield "hello!"


@pytest.fixture
def reply_squeak_content():
    yield "this is a reply!"


@pytest.fixture
def squeak_and_secret_key(private_key, squeak_content, block_info):
    yield make_squeak_with_block(
        private_key,
        squeak_content,
        block_info.block_height,
        block_info.block_hash,
    )


@pytest.fixture
def reply_squeak_and_secret_key(private_key, reply_squeak_content, block_info, squeak_hash):
    yield make_squeak_with_block(
        private_key,
        reply_squeak_content,
        block_info.block_height,
        block_info.block_hash,
        replyto_hash=squeak_hash,
    )


@pytest.fixture
def private_squeak_and_secret_key(private_key, squeak_content, block_info, recipient_public_key):
    yield make_squeak_with_block(
        private_key,
        squeak_content,
        block_info.block_height,
        block_info.block_hash,
        recipient_public_key=recipient_public_key,
    )


@pytest.fixture
def squeak(squeak_and_secret_key):
    squeak, _ = squeak_and_secret_key
    yield squeak


@pytest.fixture
def secret_key(squeak_and_secret_key):
    _, secret_key = squeak_and_secret_key
    yield secret_key


@pytest.fixture
def squeak_hash(squeak):
    yield get_hash(squeak)


@pytest.fixture
def squeak_hash_str(squeak_hash):
    yield squeak_hash.hex()


@pytest.fixture
def squeak_bytes(squeak):
    yield squeak.serialize()


@pytest.fixture
def squeak_time(squeak):
    yield squeak.nTime


@pytest.fixture
def squeak_reply_to_hash(squeak):
    yield None


@pytest.fixture
def reply_squeak(reply_squeak_and_secret_key):
    squeak, _ = reply_squeak_and_secret_key
    yield squeak


@pytest.fixture
def reply_squeak_hash(reply_squeak):
    yield get_hash(reply_squeak)


@pytest.fixture
def private_squeak(private_squeak_and_secret_key):
    squeak, _ = private_squeak_and_secret_key
    yield squeak


@pytest.fixture
def resqueak(private_key, squeak_hash, block_info):
    yield make_resqueak_with_block(
        private_key,
        squeak_hash,
        block_info.block_height,
        block_info.block_hash,
    )


@pytest.fixture
def resqueak_hash(resqueak):
    yield get_hash(resqueak)


@pytest.fixture
def peer_address():
    yield PeerAddress(
        network=Network.IPV4,
        host="fake_host",
        port=8765,
    )


@pytest.fixture
def signing_profile_name():
    yield "fake_signing_profile_name"


@pytest.fixture
def contact_profile_name():
    yield "fake_contact_profile_name"


@pytest.fixture
def recipient_contact_profile_name():
    yield "recipient_contact_profile_name"


@pytest.fixture
def recipient_signing_profile_name():
    yield "recipient_signing_profile_name"


@pytest.fixture
def signing_profile(signing_profile_name, private_key):
    yield gen_signing_profile(
        signing_profile_name,
        private_key,
    )


@pytest.fixture
def contact_profile(contact_profile_name, public_key):
    yield gen_contact_profile(
        contact_profile_name,
        public_key,
    )


@pytest.fixture
def recipient_contact_profile(recipient_contact_profile_name, recipient_public_key):
    yield gen_contact_profile(
        recipient_contact_profile_name,
        recipient_public_key,
    )


@pytest.fixture
def recipient_signing_profile(recipient_signing_profile_name, recipient_private_key):
    yield gen_signing_profile(
        recipient_signing_profile_name,
        recipient_private_key,
    )


@pytest.fixture
def squeak_entry_locked(
        squeak,
        squeak_bytes,
        squeak_hash,
        public_key,
        recipient_public_key,
        block_count,
        block_hash,
        block_time,
        squeak_time,
        squeak_reply_to_hash,
        signing_profile,
        recipient_contact_profile,
):
    yield SqueakEntry(
        squeak_hash=squeak_hash,
        serialized_squeak=squeak_bytes,
        public_key=public_key,
        recipient_public_key=recipient_public_key,
        block_height=block_count,
        block_hash=block_hash,
        block_time=block_time,
        squeak_time=squeak_time,
        reply_to=squeak_reply_to_hash,
        is_unlocked=False,
        secret_key=None,
        squeak_profile=signing_profile,
        recipient_squeak_profile=recipient_contact_profile,
        liked_time_ms=None,
        num_replies=0,
        num_resqueaks=0,
        content=None,
    )


@pytest.fixture
def peer_name():
    yield "fake_peer_name"


@pytest.fixture
def peer(peer_name, peer_address):
    yield SqueakPeer(
        peer_id=None,
        peer_name=peer_name,
        address=peer_address,
        autoconnect=False,
        share_for_free=False,
    )


@pytest.fixture
def lightning_address():
    return LightningAddressHostPort(host="my_lightning_host", port=8765)


@pytest.fixture
def external_lightning_address():
    return LightningAddressHostPort(host="my_external_lightning_host", port=13579)


@pytest.fixture
def price_msat():
    return 777


@pytest.fixture
def nonce():
    yield generate_tweak()


@pytest.fixture
def preimage(secret_key, nonce):
    yield add_tweak(secret_key, nonce)


@pytest.fixture
def payment_point(secret_key):
    yield payment_point_bytes_from_scalar_bytes(secret_key)


@pytest.fixture
def payment_hash(preimage):
    # TODO: When PTLC is used, this should be the payment point of preimage.
    yield sha256(preimage)


@pytest.fixture
def payment_request():
    yield "fake_payment_request"


@pytest.fixture
def creation_date():
    yield 777777


@pytest.fixture
def expiry():
    yield 5555


@pytest.fixture
def seller_pubkey():
    yield "fake_seller_pubkey"


@pytest.fixture
def uris(seller_pubkey, lightning_address):
    yield [
        '{}@{}:{}'.format(
            seller_pubkey,
            lightning_address.host,
            lightning_address.port,
        ),
        'fake_pubkey@foobar.com:12345',
        'fake_pubkey@fakehost.com:56789',
    ]


@pytest.fixture
def settle_index():
    yield 345


@pytest.fixture
def sent_offer(
        squeak_hash,
        price_msat,
        payment_hash,
        nonce,
        creation_date,
        expiry,
        payment_request,
        peer_address,
):
    yield SentOffer(
        sent_offer_id=None,
        squeak_hash=squeak_hash,
        payment_hash=payment_hash,
        nonce=nonce,
        price_msat=price_msat,
        payment_request=payment_request,
        invoice_time=creation_date,
        invoice_expiry=expiry,
        peer_address=peer_address,
        paid=False,
    )


@pytest.fixture
def received_offer(
        squeak_hash,
        price_msat,
        payment_hash,
        nonce,
        payment_point,
        creation_date,
        expiry,
        payment_request,
        seller_pubkey,
        lightning_address,
        peer_address,
):
    yield ReceivedOffer(
        received_offer_id=None,
        squeak_hash=squeak_hash,
        price_msat=price_msat,
        payment_hash=payment_hash,
        nonce=nonce,
        payment_point=payment_point,
        invoice_timestamp=creation_date,
        invoice_expiry=expiry,
        payment_request=payment_request,
        destination=seller_pubkey,
        lightning_address=lightning_address,
        peer_address=peer_address,
        paid=False,
    )


@pytest.fixture
def received_payment(
        squeak_hash,
        payment_hash,
        price_msat,
        settle_index,
        peer_address,
):
    yield ReceivedPayment(
        received_payment_id=None,
        created_time_ms=None,
        squeak_hash=squeak_hash,
        payment_hash=payment_hash,
        price_msat=price_msat,
        settle_index=settle_index,
        peer_address=peer_address,
    )


@pytest.fixture
def sent_payment(
        squeak_hash,
        payment_hash,
        secret_key,
        price_msat,
        seller_pubkey,
        peer_address,
):
    yield SentPayment(
        sent_payment_id=None,
        created_time_ms=None,
        peer_address=peer_address,
        squeak_hash=squeak_hash,
        payment_hash=payment_hash,
        secret_key=secret_key,
        price_msat=price_msat,
        node_pubkey=seller_pubkey,
        valid=True,
    )


@pytest.fixture
def offer(
        squeak_hash,
        nonce,
        payment_request,
        lightning_address,
):
    yield Offer(
        squeak_hash=squeak_hash,
        nonce=nonce,
        payment_request=payment_request,
        host=lightning_address.host,
        port=lightning_address.port,
    )


@pytest.fixture
def num_received_payments():
    yield 574


@pytest.fixture
def num_sent_payments():
    yield 189


@pytest.fixture
def total_amount_received_msat():
    yield 3259084


@pytest.fixture
def total_amount_sent_msat():
    yield 198374


@pytest.fixture
def received_payment_summary(
        num_received_payments,
        total_amount_received_msat,
):
    yield ReceivedPaymentSummary(
        num_received_payments=num_received_payments,
        total_amount_received_msat=total_amount_received_msat,
    )


@pytest.fixture
def sent_payment_summary(
        num_sent_payments,
        total_amount_sent_msat,
):
    yield SentPaymentSummary(
        num_sent_payments=num_sent_payments,
        total_amount_sent_msat=total_amount_sent_msat,
    )


@pytest.fixture
def payment_summary(
        received_payment_summary,
        sent_payment_summary,
):
    yield PaymentSummary(
        sent_payment_summary=sent_payment_summary,
        received_payment_summary=received_payment_summary,
    )


@pytest.fixture
def download_result():
    yield DownloadResult(
        number_downloaded=1,
        number_requested=20,
        elapsed_time_ms=56789,
        number_peers=4,
    )


@pytest.fixture
def user_config():
    yield UserConfig(
        username="default_user",
    )


@pytest.fixture
def twitter_bearer_token():
    yield 'abcdefg987654321'
