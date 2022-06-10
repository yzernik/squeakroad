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
import hashlib
import os
import random
import uuid

from bitcoin.core import CBlockHeader
from squeak.core.keys import SqueakPrivateKey

from squeaknode.core.peer_address import Network
from squeaknode.core.peer_address import PeerAddress
from squeaknode.core.peers import create_saved_peer
from squeaknode.core.profiles import create_contact_profile
from squeaknode.core.profiles import create_signing_profile
from squeaknode.core.received_payment import ReceivedPayment
from squeaknode.core.sent_payment import SentPayment
from squeaknode.core.squeaks import HASH_LENGTH
from squeaknode.core.squeaks import make_squeak_with_block


def gen_private_key():
    return SqueakPrivateKey.generate()


def gen_random_hash():
    return os.urandom(HASH_LENGTH)


def sha256(data):
    return hashlib.sha256(data).digest()


def public_key_from_private_key(private_key):
    return private_key.get_public_key()


def gen_pubkey():
    private_key = gen_private_key()
    return public_key_from_private_key(private_key)


def gen_squeak_pubkeys(n):
    return [gen_pubkey() for i in range(n)]


def gen_squeak(private_key, block_height, replyto_hash=None):
    random_content = "random_content_{}".format(uuid.uuid1())
    random_hash = gen_random_hash()
    squeak, secret_key = make_squeak_with_block(
        private_key,
        random_content,
        block_height,
        random_hash,
        replyto_hash=replyto_hash,
    )
    return squeak


def gen_block_header(block_height):
    return CBlockHeader(
        nTime=block_height * 10,  # So that block times are increasing.
    )


def gen_squeak_with_block_header(private_key, block_height, replyto_hash=None):
    """ Return a tuple with a CSqueak and a CBlockHeader.
    """
    squeak = gen_squeak(
        private_key=private_key,
        block_height=block_height,
        replyto_hash=replyto_hash,
    )
    block_info = gen_block_header(
        block_height=block_height,
    )
    return squeak, block_info


def gen_signing_profile(profile_name, private_key):
    return create_signing_profile(
        profile_name,
        private_key,
    )


def gen_contact_profile(profile_name, address):
    return create_contact_profile(
        profile_name,
        address,
    )


def gen_squeak_peer(peer_name):
    host = "random_host_{}".format(uuid.uuid1())
    port = random.randint(1, 10000)
    peer_address = PeerAddress(
        network=Network.IPV4,
        host=host,
        port=port,
    )
    return create_saved_peer(
        peer_name,
        peer_address,
        0,
    )


def gen_sent_payment(peer_address, squeak_hash, secret_key, price_msat, seller_pubkey):
    payment_hash = gen_random_hash()
    return SentPayment(
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


def gen_received_payment(peer_address, squeak_hash, price_msat, settle_index):
    payment_hash = gen_random_hash()
    return ReceivedPayment(
        received_payment_id=None,
        created_time_ms=None,
        squeak_hash=squeak_hash,
        payment_hash=payment_hash,
        price_msat=price_msat,
        settle_index=settle_index,
        peer_address=peer_address,
    )
