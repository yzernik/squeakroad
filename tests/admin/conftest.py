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

from proto import squeak_admin_pb2
from squeaknode.admin.messages import DEFAULT_PROFILE_IMAGE
from squeaknode.admin.profile_image_util import bytes_to_base64_string


@pytest.fixture
def peer_address_message():
    yield squeak_admin_pb2.PeerAddress(
        network="IPV4",
        host="fake_host",
        port=8765,
    )


@pytest.fixture
def default_profile_image():
    yield DEFAULT_PROFILE_IMAGE


@pytest.fixture
def signing_profile_msg(
        public_key,
        signing_profile_name,
        default_profile_image,
):
    img_base64_str = bytes_to_base64_string(default_profile_image)
    yield squeak_admin_pb2.SqueakProfile(
        profile_id=None,
        profile_name=signing_profile_name,
        has_private_key=True,
        pubkey=public_key.to_bytes().hex(),
        following=True,
        profile_image=img_base64_str,
        has_custom_profile_image=False,
    )


@pytest.fixture
def recipient_profile_msg(
        recipient_public_key,
        recipient_contact_profile_name,
        default_profile_image,
):
    img_base64_str = bytes_to_base64_string(default_profile_image)
    yield squeak_admin_pb2.SqueakProfile(
        profile_id=None,
        profile_name=recipient_contact_profile_name,
        has_private_key=False,
        pubkey=recipient_public_key.to_bytes().hex(),
        following=True,
        profile_image=img_base64_str,
        has_custom_profile_image=False,
    )


@pytest.fixture
def squeak_entry_msg_locked(
        squeak,
        squeak_bytes,
        squeak_hash,
        public_key,
        block_count,
        block_hash,
        block_time,
        squeak_time,
        squeak_reply_to_hash,
        signing_profile_msg,
        recipient_public_key,
        recipient_profile_msg,
):
    yield squeak_admin_pb2.SqueakDisplayEntry(
        squeak_hash=squeak_hash.hex(),
        serialized_squeak_hex=squeak_bytes.hex(),
        is_unlocked=False,
        secret_key_hex="",
        content_str=None,  # type: ignore
        block_height=block_count,
        block_hash=block_hash.hex(),
        block_time=block_time,
        squeak_time=squeak_time,
        is_reply=False,
        reply_to=squeak_reply_to_hash,  # type: ignore
        author_pubkey=public_key.to_bytes().hex(),
        is_author_known=True,
        author=signing_profile_msg,
        liked_time_ms=None,  # type: ignore
        is_private=True,
        recipient_pubkey=recipient_public_key.to_bytes().hex(),
        is_recipient_known=True,
        recipient=recipient_profile_msg,
    )


@pytest.fixture
def peer_msg(
        peer_name,
        peer_address_message,
):
    yield squeak_admin_pb2.SqueakPeer(
        peer_id=None,
        peer_name=peer_name,
        peer_address=peer_address_message,
        autoconnect=False,
    )


@pytest.fixture
def sent_offer_msg(
        squeak_hash,
        payment_hash,
        price_msat,
):
    return squeak_admin_pb2.SentOffer(
        sent_offer_id=None,
        squeak_hash=squeak_hash.hex(),
        payment_hash=payment_hash.hex(),
        price_msat=price_msat,
    )


@pytest.fixture
def received_offer_msg(
        squeak_hash,
        price_msat,
        creation_date,
        expiry,
        payment_request,
        seller_pubkey,
        lightning_address,
        peer_address_message,
):
    yield squeak_admin_pb2.OfferDisplayEntry(
        offer_id=None,
        squeak_hash=squeak_hash.hex(),
        price_msat=price_msat,
        node_pubkey=seller_pubkey,
        node_host=lightning_address.host,
        node_port=lightning_address.port,
        invoice_timestamp=creation_date,
        invoice_expiry=expiry,
        peer_address=peer_address_message,
    )


@pytest.fixture
def received_payment_msg(
        squeak_hash,
        payment_hash,
        price_msat,
        peer_address_message,
):
    yield squeak_admin_pb2.ReceivedPayment(
        received_payment_id=None,
        squeak_hash=squeak_hash.hex(),
        payment_hash=payment_hash.hex(),
        price_msat=price_msat,
        time_ms=None,
        peer_address=peer_address_message,
    )


@pytest.fixture
def sent_payment_msg(
        squeak_hash,
        payment_hash,
        secret_key,
        price_msat,
        seller_pubkey,
        peer_address_message,
):
    yield squeak_admin_pb2.SentPayment(
        sent_payment_id=None,
        squeak_hash=squeak_hash.hex(),
        payment_hash=payment_hash.hex(),
        price_msat=price_msat,
        node_pubkey=seller_pubkey,
        valid=True,
        time_ms=None,
        peer_address=peer_address_message,
    )


@pytest.fixture
def payment_summary_msg(
        num_received_payments,
        num_sent_payments,
        total_amount_received_msat,
        total_amount_sent_msat,
):
    yield squeak_admin_pb2.PaymentSummary(
        num_received_payments=num_received_payments,
        num_sent_payments=num_sent_payments,
        amount_earned_msat=total_amount_received_msat,
        amount_spent_msat=total_amount_sent_msat,
    )


@pytest.fixture
def download_result_msg(
        download_result,
):
    yield squeak_admin_pb2.DownloadResult(
        number_downloaded=download_result.number_downloaded,
        number_requested=download_result.number_requested,
        elapsed_time_ms=download_result.elapsed_time_ms,
        number_peers=download_result.number_peers,
    )
