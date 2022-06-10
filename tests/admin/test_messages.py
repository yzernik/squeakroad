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
from squeaknode.admin.messages import download_result_to_message
from squeaknode.admin.messages import message_to_peer_address
from squeaknode.admin.messages import message_to_received_payment
from squeaknode.admin.messages import message_to_sent_payment
from squeaknode.admin.messages import message_to_squeak_entry
from squeaknode.admin.messages import optional_received_offer_to_message
from squeaknode.admin.messages import optional_sent_payment_to_message
from squeaknode.admin.messages import optional_squeak_entry_to_message
from squeaknode.admin.messages import optional_squeak_hash_to_hex
from squeaknode.admin.messages import optional_squeak_peer_to_message
from squeaknode.admin.messages import optional_squeak_profile_to_message
from squeaknode.admin.messages import payment_summary_to_message
from squeaknode.admin.messages import peer_address_to_message
from squeaknode.admin.messages import received_offer_to_message
from squeaknode.admin.messages import received_payment_to_message
from squeaknode.admin.messages import sent_offer_to_message
from squeaknode.admin.messages import sent_payment_to_message
from squeaknode.admin.messages import squeak_entry_to_message
from squeaknode.admin.messages import squeak_peer_to_message
from squeaknode.admin.messages import squeak_profile_to_message


def test_peer_address_to_message(peer_address, peer_address_message):
    msg = peer_address_to_message(peer_address)

    assert msg == peer_address_message


def test_msg_to_peer_address(peer_address, peer_address_message):
    address = message_to_peer_address(peer_address_message)

    assert address == peer_address


def test_squeak_entry_to_message(squeak_entry_locked, squeak_entry_msg_locked):
    msg = squeak_entry_to_message(squeak_entry_locked)

    assert msg == squeak_entry_msg_locked


def test_message_to_squeak_entry(squeak_entry_locked, squeak_entry_msg_locked):
    entry = message_to_squeak_entry(squeak_entry_msg_locked)

    # TODO: remove this line after implementing "message_to_squeak_profile"
    entry_with_null_profile = squeak_entry_locked\
        ._replace(squeak_profile=None)\
        ._replace(recipient_public_key=None)\
        ._replace(recipient_squeak_profile=None)
    assert entry == entry_with_null_profile


def test_profile_to_message(signing_profile, signing_profile_msg):
    msg = squeak_profile_to_message(signing_profile)

    assert msg == signing_profile_msg


def test_peer_to_message(peer, peer_msg):
    msg = squeak_peer_to_message(peer)

    assert msg == peer_msg


def test_sent_offer_to_message(sent_offer, sent_offer_msg):
    msg = sent_offer_to_message(sent_offer)

    assert msg == sent_offer_msg


def test_received_offer_to_message(received_offer, received_offer_msg):
    msg = received_offer_to_message(received_offer)

    assert msg == received_offer_msg


def test_received_payment_to_message(received_payment, received_payment_msg):
    msg = received_payment_to_message(received_payment)

    assert msg == received_payment_msg


def test_message_to_received_payment(received_payment, received_payment_msg):
    decoded_received_payment = message_to_received_payment(
        received_payment_msg)

    # TODO: remove this line after including settle index in received payment msg.
    received_payment_with_empty_settle_index = received_payment._replace(
        settle_index=0)
    assert decoded_received_payment == received_payment_with_empty_settle_index


def test_download_result_to_message(download_result, download_result_msg):
    msg = download_result_to_message(download_result)

    assert msg == download_result_msg


def test_sent_payment_to_message(sent_payment, sent_payment_msg):
    msg = sent_payment_to_message(sent_payment)

    assert msg == sent_payment_msg


def test_message_to_sent_payment(sent_payment, sent_payment_msg):
    decoded_sent_payment = message_to_sent_payment(sent_payment_msg)

    # TODO: remove this line after including secret key in sent payment msg.
    sent_payment_with_empty_secret_key = sent_payment._replace(secret_key=b'')
    assert decoded_sent_payment == sent_payment_with_empty_secret_key


def test_payment_summary_to_message(
        payment_summary,
        payment_summary_msg,
):
    msg = payment_summary_to_message(
        payment_summary,
    )

    assert msg == payment_summary_msg


def test_optional_profile_to_message_none():
    msg = optional_squeak_profile_to_message(None)

    assert msg is None


def test_optional_profile_to_message(signing_profile, signing_profile_msg):
    msg = optional_squeak_profile_to_message(signing_profile)

    assert msg == signing_profile_msg


def test_optional_squeak_hash_to_str_none():
    msg = optional_squeak_hash_to_hex(None)

    assert msg is None


def test_optional_squeak_hash_to_str(squeak_hash, squeak_hash_str):
    msg = optional_squeak_hash_to_hex(squeak_hash)

    assert msg == squeak_hash_str


def test_optional_squeak_entry_to_message_none():
    msg = optional_squeak_entry_to_message(None)

    assert msg is None


def test_optional_squeak_entry_to_message(squeak_entry_locked, squeak_entry_msg_locked):
    msg = optional_squeak_entry_to_message(squeak_entry_locked)

    assert msg == squeak_entry_msg_locked


def test_optional_peer_to_message_none():
    msg = optional_squeak_peer_to_message(None)

    assert msg is None


def test_optional_peer_to_message(peer, peer_msg):
    msg = optional_squeak_peer_to_message(peer)

    assert msg == peer_msg


def test_optional_received_offer_to_message_none():
    msg = optional_received_offer_to_message(None)

    assert msg is None


def test_optional_received_offer_to_message(received_offer, received_offer_msg):
    msg = optional_received_offer_to_message(received_offer)

    assert msg == received_offer_msg


def test_optional_sent_payment_to_message_none():
    msg = optional_sent_payment_to_message(None)

    assert msg is None


def test_optional_sent_payment_to_message(sent_payment, sent_payment_msg):
    msg = optional_sent_payment_to_message(sent_payment)

    assert msg == sent_payment_msg
