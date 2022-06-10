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

from squeaknode.core.lightning_address import LightningAddressHostPort
from squeaknode.core.squeak_core import SqueakCore
from squeaknode.db.squeak_db import SqueakDb
from squeaknode.node.squeak_store import SqueakStore


@pytest.fixture
def squeak_db():
    return mock.Mock(spec=SqueakDb)


@pytest.fixture
def squeak_core():
    return mock.Mock(spec=SqueakCore)


@pytest.fixture
def lightning_host_port():
    return LightningAddressHostPort(host="my_lightning_host", port=8765)


@pytest.fixture
def price_msat():
    return 777


@pytest.fixture
def max_squeaks():
    return 55


@pytest.fixture
def max_squeaks_per_public_key_per_block():
    return 10


@pytest.fixture
def squeak_retention_s():
    return 360000


@pytest.fixture
def received_offer_retention_s():
    return 3600


@pytest.fixture
def sent_offer_retention_s():
    return 7200


@pytest.fixture
def inserted_signing_profile_id(squeak_db, signing_profile):
    yield squeak_db.insert_profile(signing_profile)


@pytest.fixture
def squeak_store(
    squeak_db,
    squeak_core,
    max_squeaks,
    max_squeaks_per_public_key_per_block,
    squeak_retention_s,
    received_offer_retention_s,
    sent_offer_retention_s,
):
    return SqueakStore(
        squeak_db,
        squeak_core,
        max_squeaks,
        max_squeaks_per_public_key_per_block,
        squeak_retention_s,
        received_offer_retention_s,
        sent_offer_retention_s,
    )


def test_save_squeak(squeak_store, squeak_db, squeak_core, block_header, squeak, squeak_hash):
    with mock.patch.object(squeak_db, 'get_number_of_squeaks', autospec=True) as mock_get_number_of_squeaks, \
            mock.patch.object(squeak_db, 'number_of_squeaks_with_public_key_with_block_height', autospec=True) as mock_number_of_squeaks_with_public_key_with_block_height, \
            mock.patch.object(squeak_db, 'insert_squeak', autospec=True) as mock_insert_squeak, \
            mock.patch.object(squeak_store.new_squeak_listener, 'handle_new_item', autospec=True) as mock_handle_new_squeak, \
            mock.patch.object(squeak_core, 'get_block_header', autospec=True) as mock_get_block_header:
        mock_get_number_of_squeaks.return_value = 0
        mock_number_of_squeaks_with_public_key_with_block_height.return_value = 0
        mock_get_block_header.return_value = block_header
        mock_insert_squeak.return_value = squeak_hash
        squeak_store.save_squeak(squeak)

        mock_insert_squeak.assert_called_once_with(squeak, block_header)
        mock_handle_new_squeak.assert_called_once_with(squeak)


def test_save_squeak_above_max(squeak_store, squeak_db, squeak_core, block_header, squeak, squeak_hash, max_squeaks):
    with mock.patch.object(squeak_db, 'get_number_of_squeaks', autospec=True) as mock_get_number_of_squeaks, \
            mock.patch.object(squeak_db, 'number_of_squeaks_with_public_key_with_block_height', autospec=True) as mock_number_of_squeaks_with_public_key_with_block_height, \
            mock.patch.object(squeak_db, 'insert_squeak', autospec=True) as mock_insert_squeak, \
            mock.patch.object(squeak_store.new_squeak_listener, 'handle_new_item', autospec=True) as mock_handle_new_squeak, \
            mock.patch.object(squeak_core, 'get_block_header', autospec=True) as mock_get_block_header:
        mock_get_number_of_squeaks.return_value = max_squeaks + 1
        mock_number_of_squeaks_with_public_key_with_block_height.return_value = 0
        mock_get_block_header.return_value = block_header
        mock_insert_squeak.return_value = squeak_hash

        with pytest.raises(Exception):
            squeak_store.save_squeak(squeak)

        assert mock_insert_squeak.call_count == 0
        assert mock_handle_new_squeak.call_count == 0


def test_save_squeak_above_max_per_pubkey(squeak_store, squeak_db, squeak_core, block_header, squeak, squeak_hash, max_squeaks_per_public_key_per_block):
    with mock.patch.object(squeak_db, 'get_number_of_squeaks', autospec=True) as mock_get_number_of_squeaks, \
            mock.patch.object(squeak_db, 'number_of_squeaks_with_public_key_with_block_height', autospec=True) as mock_number_of_squeaks_with_public_key_with_block_height, \
            mock.patch.object(squeak_db, 'insert_squeak', autospec=True) as mock_insert_squeak, \
            mock.patch.object(squeak_store.new_squeak_listener, 'handle_new_item', autospec=True) as mock_handle_new_squeak, \
            mock.patch.object(squeak_core, 'get_block_header', autospec=True) as mock_get_block_header:
        mock_get_number_of_squeaks.return_value = 0
        mock_number_of_squeaks_with_public_key_with_block_height.return_value = max_squeaks_per_public_key_per_block + 1
        mock_get_block_header.return_value = block_header
        mock_insert_squeak.return_value = squeak_hash

        with pytest.raises(Exception):
            squeak_store.save_squeak(squeak)

        assert mock_insert_squeak.call_count == 0
        assert mock_handle_new_squeak.call_count == 0


def test_save_secret_key(squeak_store, squeak_db, squeak_core, squeak, squeak_hash, secret_key):
    with mock.patch.object(squeak_db, 'get_squeak', autospec=True) as mock_get_squeak, \
            mock.patch.object(squeak_db, 'set_squeak_secret_key', autospec=True) as mock_set_squeak_secret_key, \
            mock.patch.object(squeak_store, 'unlock_squeak', autospec=True) as mock_unlock_squeak, \
            mock.patch.object(squeak_store.new_secret_key_listener, 'handle_new_item', autospec=True) as mock_handle_new_secret_key:
        mock_get_squeak.return_value = squeak
        squeak_store.save_secret_key(squeak_hash, secret_key)

        mock_set_squeak_secret_key.assert_called_once_with(
            squeak_hash, secret_key)
        mock_handle_new_secret_key.assert_called_once_with(squeak)
        mock_unlock_squeak.assert_called_once_with(squeak_hash)


def test_unlock_squeak(squeak_store, squeak_db, squeak_core, squeak, squeak_hash, secret_key, squeak_content):
    with mock.patch.object(squeak_db, 'get_squeak', autospec=True) as mock_get_squeak, \
            mock.patch.object(squeak_db, 'get_squeak_secret_key', autospec=True) as mock_get_squeak_secret_key, \
            mock.patch.object(squeak_db, 'set_squeak_decrypted_content', autospec=True) as mock_set_squeak_decrypted_content, \
            mock.patch.object(squeak_core, 'get_decrypted_content', autospec=True) as mock_get_decrypted_content:
        mock_get_squeak.return_value = squeak
        mock_get_squeak_secret_key.return_value = secret_key
        mock_get_decrypted_content.return_value = squeak_content
        squeak_store.unlock_squeak(squeak_hash)

        mock_set_squeak_decrypted_content.assert_called_once_with(
            squeak_hash, squeak_content)


# @pytest.fixture
# def unlocked_squeak(squeak_store, saved_squeak, secret_key, squeak_content):
#     saved_squeak_hash = get_hash(saved_squeak)
#     squeak_store.unlock_squeak(saved_squeak_hash, secret_key, squeak_content)
#     yield saved_squeak


# @pytest.fixture
# def deleted_squeak(squeak_store, unlocked_squeak):
#     unlocked_squeak_hash = get_hash(unlocked_squeak)
#     squeak_store.delete_squeak(unlocked_squeak_hash)
#     yield unlocked_squeak


# def test_get_squeak(squeak_store, saved_squeak):
#     saved_squeak_hash = get_hash(saved_squeak)
#     squeak_entry = squeak_store.get_squeak_entry(saved_squeak_hash)

#     assert saved_squeak == squeak_store.get_squeak(saved_squeak_hash)
#     assert squeak_entry.content is None
#     assert squeak_store.get_squeak_secret_key(saved_squeak_hash) is None


# def test_unlock_squeak(squeak_store, unlocked_squeak, squeak_content, secret_key):
#     unlocked_squeak_hash = get_hash(unlocked_squeak)
#     squeak_entry = squeak_store.get_squeak_entry(unlocked_squeak_hash)

#     assert unlocked_squeak == squeak_store.get_squeak(unlocked_squeak_hash)
#     assert squeak_entry.content == squeak_content
#     assert squeak_store.get_squeak_secret_key(
#         unlocked_squeak_hash) == secret_key


# def test_delete_squeak(squeak_store, deleted_squeak):
#     deleted_squeak_hash = get_hash(deleted_squeak)
#     squeak_entry = squeak_store.get_squeak_entry(deleted_squeak_hash)

#     assert squeak_store.get_squeak(deleted_squeak_hash) is None
#     assert squeak_entry is None
#     assert squeak_store.get_squeak_secret_key(deleted_squeak_hash) is None


def test_get_received_offer(squeak_store, squeak_db, received_offer):
    with mock.patch.object(
            squeak_db,
            'get_received_offer',
            new_callable=mock.PropertyMock,
    ) as mock_get_received_offer:
        mock_get_received_offer.return_value = received_offer

        retrieved_received_offer = squeak_store.get_received_offer(789)

    assert retrieved_received_offer == received_offer
    mock_get_received_offer.assert_called_once_with(789)


# def test_get_free_secret_key(squeak_store, squeak_core, unlocked_squeak, secret_key, peer_address):
#     unlocked_squeak_hash = get_hash(unlocked_squeak)
#     secret_key_reply = squeak_store.get_secret_key_reply(
#         unlocked_squeak_hash, peer_address, 0, None)

#     assert secret_key_reply.squeak_hash == unlocked_squeak_hash
#     assert secret_key_reply.secret_key == secret_key


# def test_get_offer_secret_key(squeak_store, squeak_core, unlocked_squeak, secret_key, peer_address, sent_offer, offer):
#     with mock.patch.object(squeak_core, 'create_offer', autospec=True) as mock_create_offer, \
#             mock.patch.object(squeak_core, 'package_offer', autospec=True) as mock_package_offer:
#         mock_create_offer.return_value = sent_offer
#         mock_package_offer.return_value = offer
#         unlocked_squeak_hash = get_hash(unlocked_squeak)
#         secret_key_reply = squeak_store.get_secret_key_reply(
#             unlocked_squeak_hash, peer_address, 1000, None)

#     assert secret_key_reply.squeak_hash == unlocked_squeak_hash
#     assert secret_key_reply.offer == offer


# def test_pay_offer(
#         squeak_store,
#         squeak_db,
#         squeak_core,
#         unlocked_squeak,
#         block_header,
#         squeak_content,
#         secret_key,
#         peer_address,
#         inserted_received_offer_id,
#         sent_payment,
# ):
#     with mock.patch.object(squeak_core, 'pay_offer', autospec=True) as mock_pay_offer, \
#             mock.patch.object(squeak_core, 'get_block_header', autospec=True) as mock_get_block_header, \
#             mock.patch.object(squeak_core, 'get_decrypted_content', autospec=True) as mock_get_decrypted_content:
#         mock_pay_offer.return_value = sent_payment
#         mock_get_block_header.return_value = block_header
#         mock_get_decrypted_content.return_value = squeak_content
#         sent_payment_id = squeak_store.pay_offer(inserted_received_offer_id)

#     retrieved_sent_payment = squeak_db.get_sent_payment(
#         sent_payment_id,
#     )

#     assert sent_payment_id is not None
#     assert retrieved_sent_payment is not None


# def test_save_received_offer_already_unlocked(
#         squeak_store,
#         unlocked_squeak,
#         offer,
#         peer_address,
# ):
#     received_offer_id = squeak_store.save_received_offer(
#         offer,
#         peer_address,
#     )

#     assert received_offer_id is None


# def test_save_received_offer(
#         squeak_store,
#         squeak_db,
#         squeak_core,
#         saved_squeak,
#         offer,
#         received_offer,
#         peer_address,
# ):
#     with mock.patch.object(squeak_core, 'unpack_offer', autospec=True) as mock_unpack_offer:
#         mock_unpack_offer.return_value = received_offer
#         received_offer_id = squeak_store.save_received_offer(
#             offer,
#             peer_address,
#         )

#     assert received_offer_id is not None
#     retrieved_received_offer = squeak_db.get_received_offer(received_offer_id)

#     assert retrieved_received_offer == received_offer._replace(
#         received_offer_id=received_offer_id,
#     )
