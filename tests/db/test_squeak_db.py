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
import time

import mock
import pytest
from sqlalchemy import create_engine

from squeaknode.core.twitter_account import TwitterAccount
from squeaknode.db.exception import SqueakDatabaseError
from squeaknode.db.squeak_db import SqueakDb
from tests.utils import gen_contact_profile
from tests.utils import gen_private_key
from tests.utils import gen_pubkey
from tests.utils import gen_random_hash
from tests.utils import gen_received_payment
from tests.utils import gen_sent_payment
from tests.utils import gen_signing_profile
from tests.utils import gen_squeak_peer
from tests.utils import gen_squeak_with_block_header


@pytest.fixture
def db_engine():
    yield create_engine('sqlite://')


@pytest.fixture
def squeak_db(db_engine):
    db = SqueakDb(db_engine)
    db.init()
    yield db


@pytest.fixture
def inserted_squeak_hash(squeak_db, squeak, block_header):
    yield squeak_db.insert_squeak(squeak, block_header)


@pytest.fixture
def inserted_reply_squeak_hash(squeak_db, reply_squeak, block_header):
    yield squeak_db.insert_squeak(reply_squeak, block_header)


@pytest.fixture
def inserted_resqueak_hash(squeak_db, resqueak, block_header):
    yield squeak_db.insert_resqueak(resqueak, block_header)


@pytest.fixture
def unlocked_squeak_hash(squeak_db, squeak, inserted_squeak_hash, secret_key, squeak_content):
    squeak_db.set_squeak_secret_key(
        inserted_squeak_hash, secret_key)
    squeak_db.set_squeak_decrypted_content(
        inserted_squeak_hash, squeak_content)
    yield inserted_squeak_hash


@pytest.fixture
def deleted_squeak_hash(squeak_db, squeak_hash):
    squeak_db.delete_squeak(
        squeak_hash,
    )
    yield squeak_hash


@pytest.fixture
def inserted_signing_profile_id(squeak_db, signing_profile):
    yield squeak_db.insert_profile(signing_profile)


@pytest.fixture
def inserted_contact_profile_id(squeak_db, contact_profile):
    yield squeak_db.insert_profile(contact_profile)


@pytest.fixture
def inserted_contact_profile(squeak_db, inserted_contact_profile_id):
    yield squeak_db.get_profile(inserted_contact_profile_id)


@pytest.fixture
def followed_contact_profile_id(squeak_db, inserted_contact_profile_id):
    squeak_db.set_profile_following(
        inserted_contact_profile_id,
        True,
    )
    yield inserted_contact_profile_id


@pytest.fixture
def unfollowed_contact_profile_id(squeak_db, followed_contact_profile_id):
    squeak_db.set_profile_following(
        followed_contact_profile_id,
        False,
    )
    yield followed_contact_profile_id


@pytest.fixture
def new_profile_name():
    yield "new_fake_profile_name"


@pytest.fixture
def profile_image_bytes():
    yield bytes.fromhex("deadbeef")


@pytest.fixture
def profile_with_new_name_id(squeak_db, inserted_contact_profile_id, new_profile_name):
    squeak_db.set_profile_name(
        inserted_contact_profile_id,
        new_profile_name,
    )
    yield inserted_contact_profile_id


@pytest.fixture
def profile_with_image_id(squeak_db, inserted_contact_profile_id, profile_image_bytes):
    squeak_db.set_profile_image(
        inserted_contact_profile_id,
        profile_image_bytes,
    )
    yield inserted_contact_profile_id


@pytest.fixture
def deleted_profile_id(squeak_db, inserted_contact_profile_id):
    squeak_db.delete_profile(
        inserted_contact_profile_id,
    )
    yield inserted_contact_profile_id


@pytest.fixture
def inserted_squeak_hashes(squeak_db, private_key):
    ret = []
    for i in range(100):
        squeak, header = gen_squeak_with_block_header(private_key, i)
        squeak_hash = squeak_db.insert_squeak(squeak, header)
        ret.append(squeak_hash)
    yield ret


@pytest.fixture
def followed_squeak_hashes(
        squeak_db,
        inserted_squeak_hashes,
        followed_contact_profile_id,
):
    yield inserted_squeak_hashes


@pytest.fixture
def authored_squeak_hashes(
        squeak_db,
        inserted_squeak_hashes,
        inserted_signing_profile_id,
):
    yield inserted_squeak_hashes


@pytest.fixture
def unfollowed_squeak_hashes(
        squeak_db,
        inserted_squeak_hashes,
        unfollowed_contact_profile_id,
):
    yield inserted_squeak_hashes


@pytest.fixture
def liked_squeak_hashes(squeak_db, inserted_squeak_hashes):
    for squeak_hash in inserted_squeak_hashes:
        squeak_db.set_squeak_liked(squeak_hash)
    yield inserted_squeak_hashes


@pytest.fixture
def unliked_squeak_hashes(squeak_db, liked_squeak_hashes):
    for squeak_hash in liked_squeak_hashes:
        squeak_db.set_squeak_unliked(squeak_hash)
    yield liked_squeak_hashes


@pytest.fixture
def liked_squeak_hash(squeak_db, inserted_squeak_hash):
    squeak_db.set_squeak_liked(inserted_squeak_hash)
    yield inserted_squeak_hash


@pytest.fixture
def unliked_squeak_hash(squeak_db, liked_squeak_hash):
    squeak_db.set_squeak_unliked(liked_squeak_hash)
    yield liked_squeak_hash


@pytest.fixture
def inserted_peer_id(squeak_db, peer):
    yield squeak_db.insert_peer(peer)


@pytest.fixture
def inserted_contact_profile_ids(squeak_db):
    ret = []
    for i in range(100):
        profile_name = "contact_profile_{}".format(i)
        public_key = gen_pubkey()
        profile = gen_contact_profile(profile_name, public_key)
        profile_id = squeak_db.insert_profile(profile)
        ret.append(profile_id)
    yield ret


@pytest.fixture
def inserted_signing_profile_ids(squeak_db):
    ret = []
    for i in range(100):
        profile_name = "signing_profile_{}".format(i)
        private_key = gen_private_key()
        profile = gen_signing_profile(profile_name, private_key)
        profile_id = squeak_db.insert_profile(profile)
        ret.append(profile_id)
    yield ret


@pytest.fixture
def followed_contact_profile_ids(squeak_db, inserted_contact_profile_ids):
    for profile_id in inserted_contact_profile_ids:
        squeak_db.set_profile_following(
            profile_id,
            True,
        )
    yield inserted_contact_profile_ids


@pytest.fixture
def inserted_squeak_peer_ids(squeak_db):
    ret = []
    for i in range(100):
        peer_name = "peer_{}".format(i)
        peer = gen_squeak_peer(peer_name)
        peer_id = squeak_db.insert_peer(peer)
        ret.append(peer_id)
    yield ret


@pytest.fixture
def autoconnect_squeak_peer_ids(squeak_db, inserted_squeak_peer_ids):
    for peer_id in inserted_squeak_peer_ids:
        squeak_db.set_peer_autoconnect(peer_id, True)
    yield inserted_squeak_peer_ids


@pytest.fixture
def new_peer_name():
    yield "new_fake_peer_name"


@pytest.fixture
def peer_with_new_name_id(squeak_db, inserted_peer_id, new_peer_name):
    squeak_db.set_peer_name(
        inserted_peer_id,
        new_peer_name,
    )
    yield inserted_peer_id


@pytest.fixture
def deleted_peer_id(squeak_db, inserted_peer_id):
    squeak_db.delete_peer(
        inserted_peer_id,
    )
    yield inserted_peer_id


@pytest.fixture
def inserted_received_offer_id(squeak_db, received_offer, creation_date):
    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = creation_date / 1000
        yield squeak_db.insert_received_offer(received_offer)


@pytest.fixture
def duplicate_inserted_received_offer_id(squeak_db, inserted_received_offer_id, received_offer):
    yield squeak_db.insert_received_offer(received_offer)


@pytest.fixture
def paid_received_offer_id(squeak_db, inserted_received_offer_id, payment_hash):
    squeak_db.set_received_offer_paid(payment_hash, True)
    yield inserted_received_offer_id


@pytest.fixture
def inserted_sent_payment_id(squeak_db, sent_payment):
    yield squeak_db.insert_sent_payment(sent_payment)


@pytest.fixture
def inserted_sent_payment_ids(
        squeak_db,
        peer_address,
        squeak_hash,
        secret_key,
        price_msat,
        seller_pubkey,
):
    ret = []
    for i in range(28):
        sent_payment = gen_sent_payment(
            peer_address,
            squeak_hash,
            secret_key,
            price_msat,
            seller_pubkey,
        )
        sent_payment_id = squeak_db.insert_sent_payment(sent_payment)
        ret.append(sent_payment_id)
    yield ret


@pytest.fixture
def inserted_sent_offer_id(squeak_db, sent_offer):
    yield squeak_db.insert_sent_offer(sent_offer)


@pytest.fixture
def paid_sent_offer_id(squeak_db, inserted_sent_offer_id, payment_hash):
    squeak_db.set_sent_offer_paid(payment_hash, True)
    yield inserted_sent_offer_id


@pytest.fixture
def inserted_received_payment_id(squeak_db, received_payment):
    yield squeak_db.insert_received_payment(received_payment)


@pytest.fixture
def duplicate_inserted_received_payment_id(squeak_db, inserted_received_payment_id, received_payment):
    yield squeak_db.insert_received_payment(received_payment)


@pytest.fixture
def inserted_received_payment_ids(
        squeak_db,
        peer_address,
        squeak_hash,
        secret_key,
        price_msat,
):
    ret = []
    for i in range(54):
        received_payment = gen_received_payment(
            peer_address,
            squeak_hash,
            price_msat,
            settle_index=i,
        )
        received_payment_id = squeak_db.insert_received_payment(
            received_payment)
        ret.append(received_payment_id)
    yield ret


@pytest.fixture
def inserted_user_config_username(squeak_db, user_config):
    yield squeak_db.insert_config(user_config)


@pytest.fixture
def duplicate_inserted_user_config_username(squeak_db, user_config, inserted_user_config_username):
    yield squeak_db.insert_config(user_config)


@pytest.fixture
def user_config_with_sell_price_msat_username(
        squeak_db,
        user_config,
        inserted_user_config_username,
        price_msat,
):
    squeak_db.set_config_sell_price_msat(
        inserted_user_config_username,
        price_msat,
    )
    yield inserted_user_config_username


@pytest.fixture
def twitter_account(inserted_signing_profile_id, twitter_bearer_token):
    yield TwitterAccount(
        twitter_account_id=None,
        handle="fake_twitter_handle",
        profile_id=inserted_signing_profile_id,
        bearer_token=twitter_bearer_token,
    )


@pytest.fixture
def inserted_twitter_account_id(squeak_db, twitter_account):
    yield squeak_db.insert_twitter_account(twitter_account)


@pytest.fixture
def duplicate_inserted_twitter_account_id(squeak_db, twitter_account, inserted_twitter_account_id):
    yield squeak_db.insert_twitter_account(twitter_account)


@pytest.fixture
def deleted_twitter_account_id(squeak_db, inserted_twitter_account_id):
    squeak_db.delete_twitter_account(
        inserted_twitter_account_id,
    )
    yield inserted_twitter_account_id


def test_init_with_retries(squeak_db):
    with mock.patch.object(squeak_db, 'init', autospec=True) as mock_init, \
            mock.patch('squeaknode.db.squeak_db.time.sleep', autospec=True) as mock_sleep:
        ret = squeak_db.init_with_retries(num_retries=5, retry_interval_s=100)

        assert ret is None
        mock_init.assert_called_once_with()
        assert mock_sleep.call_count == 0


def test_init_with_retries_fail_once(squeak_db):
    with mock.patch.object(squeak_db, 'init', autospec=True) as mock_init, \
            mock.patch('squeaknode.db.squeak_db.time.sleep', autospec=True) as mock_sleep:
        mock_init.side_effect = [Exception('some db error'), None]
        ret = squeak_db.init_with_retries(num_retries=5, retry_interval_s=100)

        assert ret is None
        mock_init.call_count == 2
        assert mock_sleep.call_count == 1


def test_init_with_retries_fail_many_times(squeak_db):
    with mock.patch.object(squeak_db, 'init', autospec=True) as mock_init, \
            mock.patch('squeaknode.db.squeak_db.time.sleep', autospec=True) as mock_sleep:
        mock_init.side_effect = [Exception('some db error')] * 5

        with pytest.raises(SqueakDatabaseError) as excinfo:
            squeak_db.init_with_retries(num_retries=5, retry_interval_s=100)
        assert "Failed to initialize database." in str(excinfo.value)

        mock_init.call_count == 5
        assert mock_sleep.call_count == 4


def test_get_squeak(squeak_db, squeak, inserted_squeak_hash):
    retrieved_squeak = squeak_db.get_squeak(inserted_squeak_hash)

    assert retrieved_squeak == squeak


def test_get_deleted_squeak(squeak_db, deleted_squeak_hash):
    retrieved_squeak = squeak_db.get_squeak(deleted_squeak_hash)

    assert retrieved_squeak is None


def test_insert_duplicate_squeak(squeak_db, squeak, block_header, inserted_squeak_hash):
    insert_result = squeak_db.insert_squeak(squeak, block_header)

    assert insert_result is None


def test_get_missing_squeak(squeak_db, squeak, squeak_hash):
    retrieved_squeak = squeak_db.get_squeak(squeak_hash)

    assert retrieved_squeak is None


def test_get_resqueak(squeak_db, resqueak, inserted_resqueak_hash):
    retrieved_resqueak = squeak_db.get_squeak(inserted_resqueak_hash)

    assert retrieved_resqueak == resqueak


def test_get_squeak_entry(
        squeak_db,
        squeak,
        block_header,
        public_key,
        signing_profile,
        inserted_squeak_hash,
        inserted_signing_profile_id,
):
    retrieved_squeak_entry = squeak_db.get_squeak_entry(inserted_squeak_hash)

    assert retrieved_squeak_entry.squeak_hash == inserted_squeak_hash
    assert retrieved_squeak_entry.public_key == public_key
    assert retrieved_squeak_entry.content is None
    assert retrieved_squeak_entry.block_time == block_header.nTime
    assert retrieved_squeak_entry.squeak_profile._replace(
        profile_id=None) == signing_profile


def test_get_private_squeak_entry(
        squeak_db,
        private_squeak,
        recipient_public_key,
        block_header,
        public_key,
        signing_profile,
        recipient_contact_profile,
        inserted_signing_profile_id,
):
    inserted_private_squeak_hash = squeak_db.insert_squeak(
        private_squeak, block_header)
    squeak_db.insert_profile(
        recipient_contact_profile)

    retrieved_squeak_entry = squeak_db.get_squeak_entry(
        inserted_private_squeak_hash)

    assert retrieved_squeak_entry.squeak_hash == inserted_private_squeak_hash
    assert retrieved_squeak_entry.public_key == public_key
    assert retrieved_squeak_entry.content is None
    assert retrieved_squeak_entry.block_time == block_header.nTime
    assert retrieved_squeak_entry.squeak_profile._replace(
        profile_id=None) == signing_profile
    assert retrieved_squeak_entry.recipient_public_key == recipient_public_key
    assert retrieved_squeak_entry.recipient_squeak_profile._replace(
        profile_id=None) == recipient_contact_profile


def test_get_missing_squeak_entry(squeak_db, squeak, squeak_hash):
    retrieved_squeak_entry = squeak_db.get_squeak_entry(squeak_hash)

    assert retrieved_squeak_entry is None


def test_get_squeak_secret_key_and_content(
        squeak_db,
        squeak,
        secret_key,
        squeak_content,
        unlocked_squeak_hash,
):
    retrieved_squeak_entry = squeak_db.get_squeak_entry(unlocked_squeak_hash)
    retrieved_secret_key = squeak_db.get_squeak_secret_key(
        unlocked_squeak_hash,
    )

    assert retrieved_squeak_entry.squeak_hash == unlocked_squeak_hash
    assert retrieved_squeak_entry.content == squeak_content
    assert retrieved_secret_key == secret_key


def test_get_squeak_secret_key_and_content_locked(
        squeak_db,
        squeak,
        secret_key,
        squeak_content,
        inserted_squeak_hash,
):
    retrieved_squeak_entry = squeak_db.get_squeak_entry(inserted_squeak_hash)
    retrieved_secret_key = squeak_db.get_squeak_secret_key(
        inserted_squeak_hash,
    )

    assert retrieved_squeak_entry.squeak_hash == inserted_squeak_hash
    assert retrieved_squeak_entry.content is None
    assert retrieved_secret_key is None


def test_get_secret_key_missing_squeak(squeak_db, squeak, squeak_hash):
    retrieved_secret_key = squeak_db.get_squeak_secret_key(
        squeak_hash,
    )

    assert retrieved_secret_key is None


def test_get_resqueak_entry(
        squeak_db,
        resqueak,
        squeak,
        block_header,
        public_key,
        signing_profile,
        inserted_resqueak_hash,
        inserted_squeak_hash,
        inserted_signing_profile_id,
):
    retrieved_resqueak_entry = squeak_db.get_squeak_entry(
        inserted_resqueak_hash)
    retrieved_squeak_entry = squeak_db.get_squeak_entry(inserted_squeak_hash)

    assert retrieved_resqueak_entry.squeak_hash == inserted_resqueak_hash
    assert retrieved_resqueak_entry.public_key == public_key
    assert retrieved_resqueak_entry.content is None
    assert retrieved_resqueak_entry.block_time == block_header.nTime
    assert retrieved_resqueak_entry.squeak_profile._replace(
        profile_id=None) == signing_profile
    assert retrieved_resqueak_entry.resqueaked_hash == inserted_squeak_hash
    assert retrieved_resqueak_entry.resqueaked_squeak == retrieved_squeak_entry
    assert retrieved_resqueak_entry.resqueaked_squeak.public_key == public_key
    assert retrieved_resqueak_entry.num_resqueaks == 0
    assert retrieved_resqueak_entry.resqueaked_squeak.num_resqueaks == 1


def test_get_timeline_squeak_entries(squeak_db, followed_squeak_hashes):
    timeline_squeak_entries = squeak_db.get_timeline_squeak_entries(
        limit=2,
        last_entry=None,
    )

    assert len(timeline_squeak_entries) == 2


def test_get_timeline_squeak_entries_all_unfollowed(squeak_db, unfollowed_squeak_hashes):
    timeline_squeak_entries = squeak_db.get_timeline_squeak_entries(
        limit=2,
        last_entry=None,
    )

    assert len(timeline_squeak_entries) == 0


def test_set_profile_following(squeak_db, followed_contact_profile_id):
    profile = squeak_db.get_profile(followed_contact_profile_id)

    assert profile.following


def test_set_profile_unfollowing(squeak_db, unfollowed_contact_profile_id):
    profile = squeak_db.get_profile(unfollowed_contact_profile_id)

    assert not profile.following


def test_set_profile_name(squeak_db, profile_with_new_name_id, new_profile_name):
    profile = squeak_db.get_profile(profile_with_new_name_id)

    assert profile.profile_name == new_profile_name


def test_set_profile_image(squeak_db, profile_with_image_id, profile_image_bytes):
    profile = squeak_db.get_profile(profile_with_image_id)

    assert profile.profile_image == profile_image_bytes


def test_deleted_profile(squeak_db, deleted_profile_id):
    profile = squeak_db.get_profile(deleted_profile_id)

    assert profile is None


def test_get_liked_squeak_entries(
        squeak_db,
        liked_squeak_hashes,
):
    # Get the liked squeak entries.
    liked_squeak_entries = squeak_db.get_liked_squeak_entries(
        limit=200,
        last_entry=None,
    )

    assert len(liked_squeak_entries) == 100


def test_get_unliked_squeak_entries(
        squeak_db,
        unliked_squeak_hashes,
):
    # Get the liked squeak entries.
    liked_squeak_entries = squeak_db.get_liked_squeak_entries(
        limit=200,
        last_entry=None,
    )

    assert len(liked_squeak_entries) == 0


def test_set_squeak_liked(squeak_db, liked_squeak_hash):
    retrieved_squeak_entry = squeak_db.get_squeak_entry(liked_squeak_hash)

    assert retrieved_squeak_entry.liked_time_ms is not None


def test_set_squeak_unliked(squeak_db, unliked_squeak_hash):
    retrieved_squeak_entry = squeak_db.get_squeak_entry(unliked_squeak_hash)

    assert retrieved_squeak_entry.liked_time_ms is None


def test_get_peer(squeak_db, peer, inserted_peer_id):
    retrieved_peer = squeak_db.get_peer(inserted_peer_id)

    assert retrieved_peer._replace(peer_id=None) == peer


def test_get_peer_missing(squeak_db):
    retrieved_peer = squeak_db.get_peer(9876)

    assert retrieved_peer is None


def test_get_peer_by_address(squeak_db, peer, inserted_peer_id, peer_address):
    retrieved_peer = squeak_db.get_peer_by_address(peer_address)

    assert retrieved_peer._replace(peer_id=None) == peer


def test_get_peer_by_address_missing(squeak_db, peer, inserted_peer_id, peer_address):
    other_address = peer_address._replace(host="other_fake_host")
    retrieved_peer = squeak_db.get_peer_by_address(other_address)

    assert retrieved_peer is None


def test_get_pubkey_squeak_entries(
        squeak_db,
        public_key,
        inserted_squeak_hashes,
):
    # Get the pubkey squeak entries.
    pubkey_squeak_entries = squeak_db.get_squeak_entries_for_public_key(
        public_key=public_key,
        limit=200,
        last_entry=None,
    )

    assert len(pubkey_squeak_entries) == len(inserted_squeak_hashes)


def test_get_pubkey_squeak_entries_other_pubkey(
        squeak_db,
        inserted_squeak_hashes,
):
    # Get the pubkey squeak entries for a different pubkey.
    other_public_key = gen_pubkey()
    pubkey_squeak_entries = squeak_db.get_squeak_entries_for_public_key(
        public_key=other_public_key,
        limit=200,
        last_entry=None,
    )

    assert len(pubkey_squeak_entries) == 0


def test_get_search_squeak_entries(
        squeak_db,
        unlocked_squeak_hash,
):
    # Get the search squeak entries.
    squeak_entries = squeak_db.get_squeak_entries_for_text_search(
        search_text="hello",
        limit=200,
        last_entry=None,
    )

    assert len(squeak_entries) == 1


def test_get_search_squeak_entries_other_text(
        squeak_db,
        unlocked_squeak_hash,
):
    # Get the search squeak entries.
    squeak_entries = squeak_db.get_squeak_entries_for_text_search(
        search_text="goodbye",
        limit=200,
        last_entry=None,
    )

    assert len(squeak_entries) == 0


def test_get_ancestor_squeak_entries(
        squeak_db,
        inserted_squeak_hash,
        inserted_reply_squeak_hash,
):
    # Get the ancestor squeak entries.
    squeak_entries = squeak_db.get_thread_ancestor_squeak_entries(
        squeak_hash=inserted_reply_squeak_hash,
    )

    assert len(squeak_entries) == 2


def test_get_ancestor_squeak_entries_no_ancestors(
        squeak_db,
        inserted_squeak_hash,
):
    # Get the ancestor squeak entries.
    squeak_entries = squeak_db.get_thread_ancestor_squeak_entries(
        squeak_hash=inserted_squeak_hash,
    )

    assert len(squeak_entries) == 1


def test_get_ancestor_squeak_entries_no_ancestors_or_root(
        squeak_db,
):
    # Get the ancestor squeak entries.
    squeak_entries = squeak_db.get_thread_ancestor_squeak_entries(
        squeak_hash=gen_random_hash(),
    )

    assert len(squeak_entries) == 0


def test_get_reply_squeak_entries(
        squeak_db,
        inserted_squeak_hash,
        inserted_reply_squeak_hash,
):
    # Get the reply squeak entries.
    squeak_entries = squeak_db.get_thread_reply_squeak_entries(
        squeak_hash=inserted_squeak_hash,
        limit=200,
        last_entry=None,
    )

    assert len(squeak_entries) == 1


def test_get_reply_squeak_entries_no_replies(
        squeak_db,
        inserted_squeak_hash,
):
    # Get the reply squeak entries.
    squeak_entries = squeak_db.get_thread_reply_squeak_entries(
        squeak_hash=inserted_squeak_hash,
        limit=200,
        last_entry=None,
    )

    assert len(squeak_entries) == 0


def test_lookup_squeaks_all(
        squeak_db,
        inserted_squeak_hashes,
):
    squeak_hashes = squeak_db.lookup_squeaks(
        public_keys=None,
        min_block=None,
        max_block=None,
        reply_to_hash=None,
        include_locked=True,
    )

    assert len(squeak_hashes) == len(inserted_squeak_hashes)


def test_lookup_squeaks_use_public_key(
        squeak_db,
        inserted_squeak_hashes,
        public_key,
):
    other_public_key = gen_pubkey()
    squeak_hashes = squeak_db.lookup_squeaks(
        public_keys=[public_key, other_public_key],
        min_block=None,
        max_block=None,
        reply_to_hash=None,
        include_locked=True,
    )

    assert len(squeak_hashes) == len(inserted_squeak_hashes)


def test_lookup_squeaks_use_pubkey_no_matches(
        squeak_db,
        inserted_squeak_hashes,
        public_key,
):
    other_public_key = gen_pubkey()
    squeak_hashes = squeak_db.lookup_squeaks(
        public_keys=[other_public_key],
        min_block=None,
        max_block=None,
        reply_to_hash=None,
        include_locked=True,
    )

    assert len(squeak_hashes) == 0


def test_lookup_squeaks_min_block(
        squeak_db,
        inserted_squeak_hashes,
):
    min_block = 35
    squeak_hashes = squeak_db.lookup_squeaks(
        public_keys=None,
        min_block=min_block,
        max_block=None,
        reply_to_hash=None,
        include_locked=True,
    )

    assert len(squeak_hashes) == len(inserted_squeak_hashes) - min_block


def test_lookup_squeaks_max_block(
        squeak_db,
        inserted_squeak_hashes,
):
    max_block = 27
    squeak_hashes = squeak_db.lookup_squeaks(
        public_keys=None,
        min_block=None,
        max_block=max_block,
        reply_to_hash=None,
        include_locked=True,
    )

    assert len(squeak_hashes) == max_block + 1


def test_lookup_squeaks_reply_to(
        squeak_db,
        inserted_squeak_hash,
        inserted_reply_squeak_hash,
):
    squeak_hashes = squeak_db.lookup_squeaks(
        public_keys=None,
        min_block=None,
        max_block=None,
        reply_to_hash=inserted_squeak_hash,
        include_locked=True,
    )

    assert len(squeak_hashes) == 1


def test_lookup_squeaks_reply_to_none(
        squeak_db,
        inserted_squeak_hash,
):
    squeak_hashes = squeak_db.lookup_squeaks(
        public_keys=None,
        min_block=None,
        max_block=None,
        reply_to_hash=inserted_squeak_hash,
        include_locked=True,
    )

    assert len(squeak_hashes) == 0


def test_lookup_squeaks_unlocked_all(
        squeak_db,
        unlocked_squeak_hash,
):
    squeak_hashes = squeak_db.lookup_squeaks(
        public_keys=None,
        min_block=None,
        max_block=None,
        reply_to_hash=None,
        include_locked=False,
    )

    assert len(squeak_hashes) == 1


def test_lookup_squeaks_unlocked_all_none(
        squeak_db,
        inserted_squeak_hash,
):
    squeak_hashes = squeak_db.lookup_squeaks(
        public_keys=None,
        min_block=None,
        max_block=None,
        reply_to_hash=None,
        include_locked=False,
    )

    assert len(squeak_hashes) == 0


def test_get_number_of_squeaks(
        squeak_db,
        inserted_squeak_hashes,
):
    num_squeaks = squeak_db.get_number_of_squeaks()

    assert num_squeaks == len(inserted_squeak_hashes)


def test_number_of_squeaks_with_public_key_with_block_height(
        squeak_db,
        public_key,
        inserted_squeak_hashes,
):
    block_height = 43
    num_squeaks = squeak_db.number_of_squeaks_with_public_key_with_block_height(
        public_key=public_key,
        block_height=block_height,
    )

    assert num_squeaks == 1


def test_get_old_squeaks_to_delete(
        squeak_db,
        followed_squeak_hashes,
):
    current_time_ms = int(time.time() * 1000)
    time_elapsed_s = 56789
    fake_current_time_ms = current_time_ms + 1000 * time_elapsed_s

    # TODO: set up test so it returns positive number of squeaks to delete.
    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        interval_s = time_elapsed_s - 10
        hashes_to_delete = squeak_db.get_old_squeaks_to_delete(
            interval_s=interval_s,
        )

        assert len(hashes_to_delete) == 100


def test_get_old_squeaks_to_delete_none(
        squeak_db,
        followed_squeak_hashes,
):
    current_time_ms = int(time.time() * 1000)
    time_elapsed_s = 56789
    fake_current_time_ms = current_time_ms + 1000 * time_elapsed_s

    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        interval_s = time_elapsed_s + 10
        hashes_to_delete = squeak_db.get_old_squeaks_to_delete(
            interval_s=interval_s,
        )

        assert len(hashes_to_delete) == 0


def test_get_old_squeaks_to_delete_none_signing_profile(
        squeak_db,
        authored_squeak_hashes,
):
    """
    `get_old_squeaks_to_delete` Method should return 0 results because
    all squeaks are authored by the signing profile.

    """
    current_time_ms = int(time.time() * 1000)
    time_elapsed_s = 56789
    fake_current_time_ms = current_time_ms + 1000 * time_elapsed_s

    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        interval_s = time_elapsed_s - 10
        hashes_to_delete = squeak_db.get_old_squeaks_to_delete(
            interval_s=interval_s,
        )

        assert len(hashes_to_delete) == 0


def test_get_old_squeaks_to_delete_none_liked(
        squeak_db,
        liked_squeak_hashes,
):
    """
    `get_old_squeaks_to_delete` Method should return 0 results because
    all squeaks are liked.

    """
    current_time_ms = int(time.time() * 1000)
    time_elapsed_s = 56789
    fake_current_time_ms = current_time_ms + 1000 * time_elapsed_s

    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        interval_s = time_elapsed_s - 10
        hashes_to_delete = squeak_db.get_old_squeaks_to_delete(
            interval_s=interval_s,
        )

        assert len(hashes_to_delete) == 0


def test_get_profiles(
        squeak_db,
        inserted_contact_profile_ids,
        inserted_signing_profile_ids,
):
    profiles = squeak_db.get_profiles()

    assert len(profiles) == len(inserted_contact_profile_ids) + \
        len(inserted_signing_profile_ids)


def test_get_signing_profiles(
        squeak_db,
        inserted_signing_profile_ids,
):
    profiles = squeak_db.get_signing_profiles()

    assert len(profiles) == len(inserted_signing_profile_ids)


def test_get_signing_profiles_none(
        squeak_db,
        inserted_contact_profile_ids,
):
    profiles = squeak_db.get_signing_profiles()

    assert len(profiles) == 0


def test_get_contact_profiles(
        squeak_db,
        inserted_contact_profile_ids,
):
    profiles = squeak_db.get_contact_profiles()

    assert len(profiles) == len(inserted_contact_profile_ids)


def test_get_contact_profiles_none(
        squeak_db,
        inserted_signing_profile_ids,
):
    profiles = squeak_db.get_contact_profiles()

    assert len(profiles) == 0


def test_get_following_profiles(
        squeak_db,
        followed_contact_profile_ids,
):
    profiles = squeak_db.get_following_profiles()

    assert len(profiles) == len(followed_contact_profile_ids)


def test_get_profile(
        squeak_db,
        inserted_signing_profile_id,
        signing_profile,
):
    profile = squeak_db.get_profile(inserted_signing_profile_id)

    assert profile == signing_profile._replace(
        profile_id=inserted_signing_profile_id)


def test_get_profile_by_public_key(
        squeak_db,
        inserted_signing_profile_id,
        signing_profile,
        public_key,
):
    profile = squeak_db.get_profile_by_public_key(public_key)

    assert profile == signing_profile._replace(
        profile_id=inserted_signing_profile_id,
    )


def test_get_profile_by_public_key_none(
        squeak_db,
        inserted_signing_profile_id,
        private_key,
):
    public_key = gen_pubkey()
    profile = squeak_db.get_profile_by_public_key(public_key)

    assert profile is None


def test_get_profile_by_name(
        squeak_db,
        inserted_signing_profile_id,
        signing_profile,
        signing_profile_name,
):
    profile = squeak_db.get_profile_by_name(signing_profile_name)

    assert profile == signing_profile._replace(
        profile_id=inserted_signing_profile_id)


def test_get_profile_by_name_none(
        squeak_db,
        inserted_signing_profile_id,
        signing_profile,
):
    other_name = "fake_name_9876"
    profile = squeak_db.get_profile_by_name(other_name)

    assert profile is None


def test_get_squeak_peers(
        squeak_db,
        inserted_squeak_peer_ids,
):
    peers = squeak_db.get_peers()

    assert len(peers) == len(inserted_squeak_peer_ids)


def test_get_autoconnect_squeak_peers(
        squeak_db,
        autoconnect_squeak_peer_ids,
):
    peers = squeak_db.get_autoconnect_peers()

    assert len(peers) == len(autoconnect_squeak_peer_ids)


def test_get_autoconnect_squeak_peers_none(
        squeak_db,
        inserted_squeak_peer_ids,
):
    peers = squeak_db.get_autoconnect_peers()

    assert len(peers) == len(inserted_squeak_peer_ids)


def test_set_peer_name(squeak_db, peer_with_new_name_id, new_peer_name):
    peer = squeak_db.get_peer(peer_with_new_name_id)

    assert peer.peer_name == new_peer_name


def test_get_deleted_peer(squeak_db, deleted_peer_id):
    retrieved_peer = squeak_db.get_peer(deleted_peer_id)

    assert retrieved_peer is None


def test_get_received_offer(squeak_db, inserted_received_offer_id, received_offer):
    retrieved_received_offer = squeak_db.get_received_offer(
        inserted_received_offer_id)

    assert retrieved_received_offer._replace(
        received_offer_id=None,
    ) == received_offer


def test_duplicate_inserted_received_offer(squeak_db, duplicate_inserted_received_offer_id):
    assert duplicate_inserted_received_offer_id is None


def test_get_received_offer_none(squeak_db, inserted_received_offer_id, received_offer):
    retrieved_received_offer = squeak_db.get_received_offer(
        -1,
    )

    assert retrieved_received_offer is None


def test_get_received_offers(squeak_db, inserted_received_offer_id, squeak_hash, creation_date, expiry):
    expire_time_s = creation_date + expiry
    current_time_s = expire_time_s - 10
    fake_current_time_ms = current_time_s * 1000

    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        received_offers = squeak_db.get_received_offers(
            squeak_hash=squeak_hash,
        )

        assert len(received_offers) == 1


def test_get_received_offers_expired_none(squeak_db, inserted_received_offer_id, squeak_hash, creation_date, expiry):
    expire_time_s = creation_date + expiry
    current_time_s = expire_time_s + 10
    fake_current_time_ms = current_time_s * 1000

    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        received_offers = squeak_db.get_received_offers(
            squeak_hash=squeak_hash,
        )

        assert len(received_offers) == 0


def test_get_received_offers_other_squeak_hash(squeak_db, inserted_received_offer_id):
    received_offers = squeak_db.get_received_offers(
        squeak_hash=gen_random_hash(),
    )

    assert len(received_offers) == 0


def test_delete_expired_received_offers(squeak_db, inserted_received_offer_id, creation_date, expiry):
    expire_time_s = creation_date + expiry
    current_time_s = expire_time_s + 10
    fake_current_time_ms = current_time_s * 1000

    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        num_deleted = squeak_db.delete_expired_received_offers(999999)

        assert num_deleted == 1


def test_delete_expired_received_offers_none(squeak_db, inserted_received_offer_id, creation_date, expiry):
    expire_time_s = creation_date + expiry
    current_time_s = expire_time_s - 10
    fake_current_time_ms = current_time_s * 1000

    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        num_deleted = squeak_db.delete_expired_received_offers(999999)

        assert num_deleted == 0


def test_delete_expired_received_offers_from_retention(squeak_db, inserted_received_offer_id, creation_date, expiry):
    expire_time_s = creation_date + expiry
    current_time_s = expire_time_s - 10
    fake_current_time_ms = current_time_s * 1000
    retention_interval_s = 10

    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        num_deleted = squeak_db.delete_expired_received_offers(
            retention_interval_s)

        assert num_deleted == 1


def test_get_received_offer_paid(squeak_db, paid_received_offer_id):
    retrieved_received_offer = squeak_db.get_received_offer(
        paid_received_offer_id,
    )

    assert retrieved_received_offer.paid


def test_get_received_offer_not_paid(squeak_db, inserted_received_offer_id):
    retrieved_received_offer = squeak_db.get_received_offer(
        inserted_received_offer_id,
    )

    assert not retrieved_received_offer.paid


def test_get_sent_payment(squeak_db, inserted_sent_payment_id, sent_payment):
    retrieved_sent_payment = squeak_db.get_sent_payment(
        inserted_sent_payment_id,
    )

    assert retrieved_sent_payment._replace(
        sent_payment_id=None,
        created_time_ms=None,
    ) == sent_payment


def test_get_sent_payment_none(squeak_db, inserted_sent_payment_id):
    retrieved_sent_payment = squeak_db.get_sent_payment(
        -1,
    )

    assert retrieved_sent_payment is None


def test_get_sent_payments(squeak_db, inserted_sent_payment_ids):
    retrieved_sent_payments = squeak_db.get_sent_payments(
        limit=1000,
        last_sent_payment=None,
    )

    assert len(retrieved_sent_payments) == len(inserted_sent_payment_ids)


def test_get_sent_payments_for_squeak(squeak_db, squeak_hash, inserted_sent_payment_ids):
    retrieved_sent_payments = squeak_db.get_sent_payments_for_squeak(
        squeak_hash,
        limit=1000,
        last_sent_payment=None,
    )

    assert len(retrieved_sent_payments) == len(inserted_sent_payment_ids)


def test_get_sent_payments_for_pubkey(squeak_db, public_key, inserted_squeak_hash, inserted_sent_payment_ids):
    retrieved_sent_payments = squeak_db.get_sent_payments_for_pubkey(
        public_key,
        limit=1000,
        last_sent_payment=None,
    )

    assert len(retrieved_sent_payments) == len(inserted_sent_payment_ids)


def test_get_sent_offer(squeak_db, inserted_sent_offer_id, sent_offer, payment_hash):
    retrieved_sent_offer = squeak_db.get_sent_offer_by_payment_hash(
        payment_hash,
    )

    assert retrieved_sent_offer._replace(
        sent_offer_id=None,
    ) == sent_offer


def test_get_sent_offer_missing(squeak_db, inserted_sent_offer_id):
    random_hash = gen_random_hash()
    retrieved_sent_offer = squeak_db.get_sent_offer_by_payment_hash(
        random_hash,
    )

    assert retrieved_sent_offer is None


def test_get_sent_offer_by_squeak_and_peer(
        squeak_db,
        inserted_sent_offer_id,
        sent_offer,
        squeak_hash,
        peer_address,
        creation_date,
        expiry,
):
    expire_time_s = creation_date + expiry
    current_time_s = expire_time_s - 10
    fake_current_time_ms = current_time_s * 1000

    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        retrieved_sent_offer = squeak_db.get_sent_offer_by_squeak_hash_and_peer(
            squeak_hash,
            peer_address,
        )

        assert retrieved_sent_offer._replace(
            sent_offer_id=None,
        ) == sent_offer


def test_get_sent_offer_by_squeak_and_peer_wrong_squeak_hash(
        squeak_db,
        inserted_sent_offer_id,
        sent_offer,
        peer_address,
        creation_date,
        expiry,
):
    expire_time_s = creation_date + expiry
    current_time_s = expire_time_s - 10
    fake_current_time_ms = current_time_s * 1000

    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        retrieved_sent_offer = squeak_db.get_sent_offer_by_squeak_hash_and_peer(
            gen_random_hash(),
            peer_address,
        )

        assert retrieved_sent_offer is None


def test_get_sent_offer_by_squeak_and_peer_wrong_peer_address(
        squeak_db,
        inserted_sent_offer_id,
        sent_offer,
        squeak_hash,
        peer_address,
        creation_date,
        expiry,
):
    expire_time_s = creation_date + expiry
    current_time_s = expire_time_s - 10
    fake_current_time_ms = current_time_s * 1000

    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        retrieved_sent_offer = squeak_db.get_sent_offer_by_squeak_hash_and_peer(
            squeak_hash,
            peer_address._replace(host="wrong_host"),
        )

        assert retrieved_sent_offer is None


def test_get_sent_offers(squeak_db, inserted_sent_offer_id):
    sent_offers = squeak_db.get_sent_offers()

    assert len(sent_offers) == 1


def test_get_sent_offers_none(squeak_db):
    sent_offers = squeak_db.get_sent_offers()

    assert len(sent_offers) == 0


def test_delete_expired_sent_offers(squeak_db, inserted_sent_offer_id, creation_date, expiry):
    expire_time_s = creation_date + expiry
    current_time_s = expire_time_s + 10
    fake_current_time_ms = current_time_s * 1000

    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        num_deleted = squeak_db.delete_expired_sent_offers(5)

        assert num_deleted == 1


def test_delete_expired_sent_offers_not_expired(squeak_db, inserted_sent_offer_id, creation_date, expiry):
    expire_time_s = creation_date + expiry
    current_time_s = expire_time_s + 10
    fake_current_time_ms = current_time_s * 1000

    with mock.patch.object(SqueakDb, 'timestamp_now_ms', new_callable=mock.PropertyMock) as mock_timestamp_ms:
        mock_timestamp_ms.return_value = fake_current_time_ms

        num_deleted = squeak_db.delete_expired_sent_offers(15)

        assert num_deleted == 0


def test_get_sent_offer_paid(squeak_db, paid_sent_offer_id, payment_hash):
    retrieved_sent_offer = squeak_db.get_sent_offer_by_payment_hash(
        payment_hash,
    )

    assert retrieved_sent_offer.paid


def test_get_sent_offer_not_paid(squeak_db, inserted_sent_offer_id, payment_hash):
    retrieved_sent_offer = squeak_db.get_sent_offer_by_payment_hash(
        payment_hash,
    )

    assert not retrieved_sent_offer.paid


def test_get_single_received_payment(squeak_db, inserted_received_payment_id, received_payment):
    received_payments = squeak_db.get_received_payments(
        limit=10,
        last_received_payment=None,
    )

    assert received_payments[0]._replace(
        received_payment_id=None,
        created_time_ms=None,
    ) == received_payment


def test_duplicate_inserted_received_payment(squeak_db, duplicate_inserted_received_payment_id):
    assert duplicate_inserted_received_payment_id is None


def test_get_received_payments(squeak_db, inserted_received_payment_ids):
    received_payments = squeak_db.get_received_payments(
        limit=1000,
        last_received_payment=None,
    )

    assert len(received_payments) == len(inserted_received_payment_ids)


def test_get_received_payments_for_squeak(squeak_db, squeak_hash, inserted_received_payment_ids):
    received_payments = squeak_db.get_received_payments_for_squeak(
        squeak_hash,
        limit=1000,
        last_received_payment=None,
    )

    assert len(received_payments) == len(inserted_received_payment_ids)


def test_get_received_payments_for_pubkey(squeak_db, public_key, inserted_squeak_hash, inserted_received_payment_ids):
    received_payments = squeak_db.get_received_payments_for_pubkey(
        public_key,
        limit=1000,
        last_received_payment=None,
    )

    assert len(received_payments) == len(inserted_received_payment_ids)


def test_yield_received_payments_from_settle_index(squeak_db, inserted_received_payment_ids):
    payments_iter = squeak_db.yield_received_payments_from_index(
        start_index=0,
    )

    assert len(list(payments_iter)) == len(inserted_received_payment_ids)


def test_yield_received_payments_from_other_settle_index(squeak_db, inserted_received_payment_ids):
    index = 43
    payments_iter = squeak_db.yield_received_payments_from_index(
        start_index=index,
    )

    assert len(list(payments_iter)) == len(
        inserted_received_payment_ids) - index


def test_get_latest_settle_index(squeak_db, inserted_received_payment_ids):
    latest_settle_index = squeak_db.get_latest_settle_index()

    assert latest_settle_index == len(inserted_received_payment_ids) - 1


def test_clear_settle_index(squeak_db, inserted_received_payment_ids):
    squeak_db.clear_received_payment_settle_indices()
    latest_settle_index = squeak_db.get_latest_settle_index()

    assert latest_settle_index == 0


def test_get_received_payment_summary(squeak_db, inserted_received_payment_ids, price_msat):
    received_payment_summary = squeak_db.get_received_payment_summary()

    assert received_payment_summary.num_received_payments == len(
        inserted_received_payment_ids)
    assert received_payment_summary.total_amount_received_msat == price_msat * \
        len(inserted_received_payment_ids)


def test_get_received_payment_summary_for_squeak(squeak_db, squeak_hash, inserted_received_payment_ids, price_msat):
    received_payment_summary = squeak_db.get_received_payment_summary_for_squeak(
        squeak_hash)

    assert received_payment_summary.num_received_payments == len(
        inserted_received_payment_ids)
    assert received_payment_summary.total_amount_received_msat == price_msat * \
        len(inserted_received_payment_ids)


def test_get_received_payment_summary_for_pubkey(squeak_db, public_key, inserted_squeak_hash, inserted_received_payment_ids, price_msat):
    received_payment_summary = squeak_db.get_received_payment_summary_for_pubkey(
        public_key)

    assert received_payment_summary.num_received_payments == len(
        inserted_received_payment_ids)
    assert received_payment_summary.total_amount_received_msat == price_msat * \
        len(inserted_received_payment_ids)


def test_get_sent_payment_summary(squeak_db, inserted_sent_payment_ids, price_msat):
    sent_payment_summary = squeak_db.get_sent_payment_summary()

    assert sent_payment_summary.num_sent_payments == len(
        inserted_sent_payment_ids)
    assert sent_payment_summary.total_amount_sent_msat == price_msat * \
        len(inserted_sent_payment_ids)


def test_get_sent_payment_summary_for_squeak(squeak_db, squeak_hash, inserted_sent_payment_ids, price_msat):
    sent_payment_summary = squeak_db.get_sent_payment_summary_for_squeak(
        squeak_hash)

    assert sent_payment_summary.num_sent_payments == len(
        inserted_sent_payment_ids)
    assert sent_payment_summary.total_amount_sent_msat == price_msat * \
        len(inserted_sent_payment_ids)


def test_get_sent_payment_summary_for_pubkey(squeak_db, public_key, inserted_squeak_hash, inserted_sent_payment_ids, price_msat):
    sent_payment_summary = squeak_db.get_sent_payment_summary_for_pubkey(
        public_key)

    assert sent_payment_summary.num_sent_payments == len(
        inserted_sent_payment_ids)
    assert sent_payment_summary.total_amount_sent_msat == price_msat * \
        len(inserted_sent_payment_ids)


def test_get_config(squeak_db, user_config, inserted_user_config_username):
    retrieved_config = squeak_db.get_config(inserted_user_config_username)

    assert retrieved_config == user_config


def test_duplicate_inserted_config(squeak_db, duplicate_inserted_user_config_username):
    assert duplicate_inserted_user_config_username is None


def test_get_config_missing(squeak_db):
    retrieved_config = squeak_db.get_config("fake_username")

    assert retrieved_config is None


def test_set_sell_price_msat(
        squeak_db,
        user_config_with_sell_price_msat_username,
        price_msat,
):
    retrieved_config = squeak_db.get_config(
        user_config_with_sell_price_msat_username)

    assert retrieved_config.sell_price_msat == price_msat


def test_get_twitter_account(
        squeak_db,
        twitter_account,
        inserted_twitter_account_id,
        signing_profile,
):
    retrieved_twitter_accounts = squeak_db.get_twitter_accounts()

    assert retrieved_twitter_accounts[0].handle == twitter_account.handle
    assert retrieved_twitter_accounts[0].profile_id == twitter_account.profile_id
    assert retrieved_twitter_accounts[0].bearer_token == twitter_account.bearer_token
    assert retrieved_twitter_accounts[0].profile._replace(profile_id=None) == \
        signing_profile


def test_duplicate_twitter_account(squeak_db, duplicate_inserted_twitter_account_id):
    assert duplicate_inserted_twitter_account_id is None


def test_get_twitter_account_all_deleted(squeak_db, twitter_account, deleted_twitter_account_id):
    retrieved_twitter_accounts = squeak_db.get_twitter_accounts()

    assert len(retrieved_twitter_accounts) == 0
