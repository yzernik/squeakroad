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
from __future__ import print_function

import datetime
import time

import pytest
from squeak.core import CheckSqueak
from squeak.core import CSqueak

from proto import lnd_pb2 as ln
from proto import squeak_admin_pb2
from tests.util import channel
from tests.util import clear_sell_price
from tests.util import create_contact_profile
from tests.util import create_saved_peer
from tests.util import create_signing_profile
from tests.util import delete_profile
from tests.util import delete_squeak
from tests.util import download_squeak
from tests.util import download_squeak_secret_key
from tests.util import free_price
from tests.util import get_balance
from tests.util import get_external_address
from tests.util import get_hash
from tests.util import get_network
from tests.util import get_peer_by_address
from tests.util import get_profile_by_pubkey
from tests.util import get_pubkey_squeak_displays
from tests.util import get_search_squeaks
from tests.util import get_sell_price
from tests.util import get_squeak_display
from tests.util import get_squeak_profile
from tests.util import import_signing_profile
from tests.util import make_squeak
from tests.util import peer_connection
from tests.util import send_coins
from tests.util import set_sell_price
from tests.util import subscribe_squeak_ancestor_entries
from tests.util import subscribe_squeak_entry


def test_get_network(admin_stub):
    # Get the network
    network = get_network(admin_stub)

    assert network == "simnet"


def test_get_sell_price(admin_stub):
    # Get the sell price
    price = get_sell_price(admin_stub)

    assert price.price_msat == 1000000

    set_sell_price(admin_stub, 98765)
    price = get_sell_price(admin_stub)

    assert price.price_msat == 98765

    clear_sell_price(admin_stub)
    price = get_sell_price(admin_stub)

    assert price.price_msat == 1000000


def test_get_external_address(admin_stub):
    # Get the external address
    external_address = get_external_address(admin_stub)

    print(external_address)
    # assert external_address.host is not None and len(external_address.host) > 0
    assert external_address.host == 'myexternaladdress.com'
    # assert external_address.port > 0
    assert external_address.port == 8765


def test_reprocess_received_payments(admin_stub):
    # Reprocess received payments
    reprocess_received_payments_response = admin_stub.ReprocessReceivedPayments(
        squeak_admin_pb2.ReprocessReceivedPaymentsRequest()
    )

    assert reprocess_received_payments_response is not None


def test_get_profile(admin_stub, signing_profile_id):
    # Get the squeak profile
    squeak_profile = get_squeak_profile(admin_stub, signing_profile_id)
    # address = squeak_profile.address
    pubkey = squeak_profile.pubkey
    name = squeak_profile.profile_name

    # Get the same squeak profile by address
    # get_squeak_profile_by_pubkey_response = admin_stub.GetSqueakProfileByAddress(
    #     squeak_admin_pb2.GetSqueakProfileByAddressRequest(
    #         pubkey=pubkey,
    #     )
    # )
    profile = get_profile_by_pubkey(admin_stub, pubkey)
    # assert address == get_squeak_profile_by_address_response.squeak_profile.address
    assert pubkey == profile.pubkey
    assert name == profile.profile_name

    # Get the same squeak profile by name
    get_squeak_profile_by_name_response = admin_stub.GetSqueakProfileByName(
        squeak_admin_pb2.GetSqueakProfileByNameRequest(
            name=name,
        )
    )
    assert pubkey == get_squeak_profile_by_name_response.squeak_profile.pubkey
    assert name == get_squeak_profile_by_name_response.squeak_profile.profile_name


def test_make_squeak(admin_stub, signing_profile_id):
    # Create a new squeak using the new profile
    make_squeak_content = "Hello from the profile on the server!"
    make_squeak_hash = make_squeak(
        admin_stub, signing_profile_id, make_squeak_content)
    assert len(make_squeak_hash) == 32 * 2

    # Get the squeak display item
    get_squeak_display_entry = get_squeak_display(
        admin_stub, make_squeak_hash)
    assert get_squeak_display_entry.squeak_hash == make_squeak_hash
    assert (
        get_squeak_display_entry.content_str == "Hello from the profile on the server!"
    )
    # assert get_squeak_display_response.squeak_display_entry.author_address == signing_profile_address
    assert get_squeak_display_entry.is_author_known
    assert get_squeak_display_entry.HasField("author")
    assert len(
        get_squeak_display_entry.author.profile_image) > 0
    assert not get_squeak_display_entry.is_reply
    assert not bool(get_squeak_display_entry.reply_to)
    assert len(get_squeak_display_entry.secret_key_hex) == 32 * 2

    # Block time should be within the past hour
    block_time = datetime.datetime.fromtimestamp(
        get_squeak_display_entry.block_time,
    )
    one_hour = datetime.timedelta(hours=1)
    assert block_time > datetime.datetime.now() - one_hour
    assert block_time < datetime.datetime.now() + one_hour

    # Get the squeak profile
    squeak_profile = get_squeak_profile(admin_stub, signing_profile_id)
    squeak_profile_pubkey = squeak_profile.pubkey
    squeak_profile_name = squeak_profile.profile_name

    # Get all squeak displays for the known address
    # get_address_squeak_display_response = admin_stub.GetAddressSqueakDisplays(
    #     squeak_admin_pb2.GetAddressSqueakDisplaysRequest(
    #         pubkey=squeak_profile_pubkey,
    #         limit=100,
    #     ),
    # )
    address_squeak_displays = get_pubkey_squeak_displays(
        admin_stub, squeak_profile_pubkey, 100)

    assert len(address_squeak_displays) == 1
    for (
        squeak_display_entry
    ) in address_squeak_displays:
        assert squeak_display_entry.author.profile_name == squeak_profile_name
        assert squeak_display_entry.author.pubkey == squeak_profile_pubkey

    # check serialized squeak hex string
    serialized_squeak_hex = get_squeak_display_entry.serialized_squeak_hex
    # print("serialized_squeak_hex: {}".format(serialized_squeak_hex))
    assert len(serialized_squeak_hex) > 200
    serialized_squeak = bytes.fromhex(serialized_squeak_hex)
    deserialized_squeak = CSqueak.deserialize(serialized_squeak)
    assert get_hash(deserialized_squeak) == make_squeak_hash
    CheckSqueak(deserialized_squeak)


def test_make_reply_squeak(
    admin_stub, saved_squeak_hash, signing_profile_id
):
    # Make another squeak as a reply
    reply_1_squeak_hash = make_squeak(
        admin_stub,
        signing_profile_id,
        "Reply #1",
        saved_squeak_hash,
    )

    # Make a second squeak as a reply
    reply_2_squeak_hash = make_squeak(
        admin_stub,
        signing_profile_id,
        "Reply #2",
        reply_1_squeak_hash,
    )

    # Get the squeak and check that the reply field is correct
    get_reply_squeak_display_entry = get_squeak_display(
        admin_stub, reply_2_squeak_hash)
    assert (
        get_reply_squeak_display_entry.squeak_hash == reply_2_squeak_hash
    )
    assert (
        get_reply_squeak_display_entry.reply_to == reply_1_squeak_hash
    )

    # Get the ancestors of the latest reply squeak
    get_ancestors_response = admin_stub.GetAncestorSqueakDisplays(
        squeak_admin_pb2.GetAncestorSqueakDisplaysRequest(
            squeak_hash=reply_2_squeak_hash,
        )
    )
    assert len(get_ancestors_response.squeak_display_entries) == 3

    # Get the replies of the original squeak
    get_replies_response = admin_stub.GetReplySqueakDisplays(
        squeak_admin_pb2.GetReplySqueakDisplaysRequest(
            squeak_hash=saved_squeak_hash,
            limit=100,
        )
    )
    assert len(get_replies_response.squeak_display_entries) == 1
    assert reply_1_squeak_hash in [
        entry.squeak_hash
        for entry in get_replies_response.squeak_display_entries
    ]


def test_make_private_squeak(admin_stub, signing_profile_id, recipient_contact_profile_id):
    make_squeak_content = "This is a private squeak!"
    make_squeak_hash = make_squeak(
        admin_stub,
        signing_profile_id,
        make_squeak_content,
        recipient_profile_id=recipient_contact_profile_id,
    )
    assert len(make_squeak_hash) == 32 * 2

    # Get the squeak display item
    get_squeak_display_entry = get_squeak_display(
        admin_stub, make_squeak_hash)
    assert get_squeak_display_entry.squeak_hash == make_squeak_hash
    assert (
        get_squeak_display_entry.content_str == "This is a private squeak!"
    )
    assert get_squeak_display_entry.is_author_known
    assert get_squeak_display_entry.HasField("author")
    assert len(
        get_squeak_display_entry.author.profile_image) > 0
    assert not get_squeak_display_entry.is_reply
    assert not bool(get_squeak_display_entry.reply_to)
    assert len(get_squeak_display_entry.secret_key_hex) == 32 * 2


def test_make_signing_profile(admin_stub):
    # Create a new signing profile
    profile_name = "test_signing_profile_name"
    profile_id = create_signing_profile(admin_stub, profile_name)

    # Get the new squeak profile
    squeak_profile = get_squeak_profile(admin_stub, profile_id)
    assert squeak_profile.profile_name == profile_name
    squeak_profile_pubkey = squeak_profile.pubkey

    # Get all signing profiles
    get_signing_profiles_response = admin_stub.GetSigningProfiles(
        squeak_admin_pb2.GetSigningProfilesRequest()
    )
    signing_profile_names = [
        profile.profile_name
        for profile in get_signing_profiles_response.squeak_profiles
    ]
    assert profile_name in signing_profile_names

    # Get squeak profile by address
    # get_profile_by_address_response = admin_stub.GetSqueakProfileByAddress(
    #     squeak_admin_pb2.GetSqueakProfileByAddressRequest(
    #         pubkey=squeak_profile_pubkey
    #     )
    # )
    profile = get_profile_by_pubkey(admin_stub, squeak_profile_pubkey)
    assert profile.profile_name == profile_name

    # Export the private key, delete the profile, and re-import it.
    get_private_key_response = admin_stub.GetSqueakProfilePrivateKey(
        squeak_admin_pb2.GetSqueakProfilePrivateKeyRequest(
            profile_id=profile_id,
        )
    )
    private_key = get_private_key_response.private_key

    delete_profile(admin_stub, profile_id)
    new_profile_id = import_signing_profile(
        admin_stub,
        "imported_profile_name",
        private_key,
    )

    # Get the new imported profile
    squeak_profile = get_squeak_profile(admin_stub, new_profile_id)
    assert squeak_profile.profile_name == "imported_profile_name"
    assert squeak_profile.pubkey == squeak_profile_pubkey


def test_make_contact_profile(admin_stub, public_key):
    # Create a new contact profile
    contact_name = "test_contact_profile_name"
    contact_profile_id = create_contact_profile(
        admin_stub, contact_name, public_key)

    # Get all contact profiles
    get_contact_profiles_response = admin_stub.GetContactProfiles(
        squeak_admin_pb2.GetContactProfilesRequest()
    )
    contact_profile_names = [
        profile.profile_name
        for profile in get_contact_profiles_response.squeak_profiles
    ]
    contact_profile_ids = [
        profile.profile_id
        for profile in get_contact_profiles_response.squeak_profiles
    ]
    assert contact_name in contact_profile_names
    assert contact_profile_id in contact_profile_ids


def test_make_signing_profile_empty_name(admin_stub):
    # Try to create a new signing profile with an empty name
    with pytest.raises(Exception) as excinfo:
        create_signing_profile(admin_stub, "")
    assert "Profile name cannot be empty." in str(excinfo.value)


def test_make_contact_profile_empty_name(admin_stub, public_key):
    # Try to create a new contact profile with an empty name
    with pytest.raises(Exception) as excinfo:
        create_contact_profile(admin_stub, "", public_key)
    assert "Profile name cannot be empty." in str(excinfo.value)


def test_set_profile_following(admin_stub, contact_profile_id):
    # Set the profile to be following
    admin_stub.SetSqueakProfileFollowing(
        squeak_admin_pb2.SetSqueakProfileFollowingRequest(
            profile_id=contact_profile_id,
            following=True,
        )
    )

    # Get the squeak profile again
    squeak_profile = get_squeak_profile(admin_stub, contact_profile_id)
    assert squeak_profile.following

    # Set the profile to be not following
    admin_stub.SetSqueakProfileFollowing(
        squeak_admin_pb2.SetSqueakProfileFollowingRequest(
            profile_id=contact_profile_id,
            following=False,
        )
    )

    # Get the squeak profile again
    squeak_profile = get_squeak_profile(admin_stub, contact_profile_id)
    assert not squeak_profile.following


def test_rename_profile(admin_stub, contact_profile_id, random_name):
    # Rename the profile to something new
    admin_stub.RenameSqueakProfile(
        squeak_admin_pb2.RenameSqueakProfileRequest(
            profile_id=contact_profile_id,
            profile_name=random_name,
        )
    )

    # Get the squeak profile
    squeak_profile = get_squeak_profile(admin_stub, contact_profile_id)
    assert squeak_profile.profile_name == random_name


def test_set_profile_image(admin_stub, contact_profile_id, random_image, random_image_base64_string):
    # print("random_image: {}".format(random_image))
    # print("random_image_base64_string: {}".format(random_image_base64_string))
    # Set the profile image to something new
    admin_stub.SetSqueakProfileImage(
        squeak_admin_pb2.SetSqueakProfileImageRequest(
            profile_id=contact_profile_id,
            profile_image=random_image_base64_string,
        )
    )

    # Get the squeak profile
    squeak_profile = get_squeak_profile(admin_stub, contact_profile_id)
    assert squeak_profile.profile_image == random_image_base64_string
    assert squeak_profile.has_custom_profile_image

    # Clear the profile image
    admin_stub.ClearSqueakProfileImage(
        squeak_admin_pb2.ClearSqueakProfileImageRequest(
            profile_id=contact_profile_id,
        )
    )

    # Get the squeak profile
    squeak_profile = get_squeak_profile(admin_stub, contact_profile_id)
    assert squeak_profile.profile_image != random_image_base64_string
    assert not squeak_profile.has_custom_profile_image


def test_delete_profile(admin_stub, random_name, public_key, contact_profile_id):
    # Delete the profile
    delete_profile(admin_stub, contact_profile_id)

    # Try to get the profile and fail
    squeak_profile = get_squeak_profile(admin_stub, contact_profile_id)
    assert squeak_profile is None

    get_squeak_profile_by_name_response = admin_stub.GetSqueakProfileByName(
        squeak_admin_pb2.GetSqueakProfileByNameRequest(
            name=random_name,
        )
    )
    assert not get_squeak_profile_by_name_response.HasField("squeak_profile")

    # get_squeak_profile_by_pubkey_response = admin_stub.GetSqueakProfileByAddress(
    #     squeak_admin_pb2.GetSqueakProfileByAddressRequest(
    #         pubkey=public_key.to_bytes().hex(),
    #     )
    # )
    profile = get_profile_by_pubkey(admin_stub, public_key.to_bytes().hex())
    # assert not get_squeak_profile_by_pubkey_response.HasField(
    #     "squeak_profile")
    assert profile is None


def test_get_profile_private_key(admin_stub, signing_profile_id):
    # Get the private key
    private_key_response = admin_stub.GetSqueakProfilePrivateKey(
        squeak_admin_pb2.GetSqueakProfilePrivateKeyRequest(
            profile_id=signing_profile_id,
        )
    )

    # print(private_key_response.private_key)
    assert len(private_key_response.private_key) > 0


def test_get_following_squeaks(
    admin_stub, saved_squeak_hash, signing_profile_id
):
    # Set the profile to be following
    admin_stub.SetSqueakProfileFollowing(
        squeak_admin_pb2.SetSqueakProfileFollowingRequest(
            profile_id=signing_profile_id,
            following=True,
        )
    )

    # Get all squeak displays for the known address
    get_timeline_squeak_display_response = admin_stub.GetTimelineSqueakDisplays(
        squeak_admin_pb2.GetTimelineSqueakDisplaysRequest(
            limit=100,
        )
    )
    assert len(get_timeline_squeak_display_response.squeak_display_entries) >= 1
    for (
        squeak_display_entry
    ) in get_timeline_squeak_display_response.squeak_display_entries:
        # TODO: check the profile id of the squeak display entry
        # assert squeak_display_entry.profile_id == signing_profile_id
        pass


def test_delete_squeak(admin_stub, saved_squeak_hash):
    # Delete the squeak
    delete_squeak(admin_stub, saved_squeak_hash)
    # Try to get the squeak display item
    squeak_display_entry = get_squeak_display(
        admin_stub, saved_squeak_hash)

    assert squeak_display_entry is None


def test_create_peer(admin_stub):
    # Add a new peer
    peer_id = create_saved_peer(
        admin_stub,
        "fake_peer_name",
        "fake_host",
        1234,
    )

    # Get the new peer
    get_peer_response = admin_stub.GetPeer(
        squeak_admin_pb2.GetPeerRequest(
            peer_id=peer_id,
        )
    )
    assert get_peer_response.squeak_peer.peer_address.host == "fake_host"
    assert get_peer_response.squeak_peer.peer_address.port == 1234

    # Get all peers
    get_peers_response = admin_stub.GetPeers(
        squeak_admin_pb2.GetPeersRequest())
    peer_hosts = [
        squeak_peer.peer_address.host
        for squeak_peer in get_peers_response.squeak_peers
    ]
    assert "fake_host" in peer_hosts

    # Get the new peer by address
    get_peer_by_address_response = get_peer_by_address(
        admin_stub,
        host="fake_host",
        port=1234,
    )
    assert get_peer_by_address_response.peer_name == "fake_peer_name"
    assert get_peer_response.squeak_peer.autoconnect


def test_set_peer_autoconnect(admin_stub, peer_id):
    # Set the peer to be autoconnect
    admin_stub.SetPeerAutoconnect(
        squeak_admin_pb2.SetPeerAutoconnectRequest(
            peer_id=peer_id,
            autoconnect=True,
        )
    )

    # Get the peer again
    get_peer_response = admin_stub.GetPeer(
        squeak_admin_pb2.GetPeerRequest(
            peer_id=peer_id,
        )
    )
    assert get_peer_response.squeak_peer.autoconnect

    # Set the peer to be not autoconnect
    admin_stub.SetPeerAutoconnect(
        squeak_admin_pb2.SetPeerAutoconnectRequest(
            peer_id=peer_id,
            autoconnect=False,
        )
    )

    # Get the peer again
    get_peer_response = admin_stub.GetPeer(
        squeak_admin_pb2.GetPeerRequest(
            peer_id=peer_id,
        )
    )
    assert not get_peer_response.squeak_peer.autoconnect


def test_rename_peer(admin_stub, peer_id, random_name):
    # Rename the peer
    admin_stub.RenamePeer(
        squeak_admin_pb2.RenamePeerRequest(
            peer_id=peer_id,
            peer_name=random_name,
        )
    )

    # Get the peer again
    get_peer_response = admin_stub.GetPeer(
        squeak_admin_pb2.GetPeerRequest(
            peer_id=peer_id,
        )
    )
    assert get_peer_response.squeak_peer.peer_name == random_name


def test_delete_peer(admin_stub, peer_id):
    # Delete the peer
    admin_stub.DeletePeer(
        squeak_admin_pb2.DeletePeerRequest(
            peer_id=peer_id,
        )
    )

    get_peer_response = admin_stub.GetPeer(
        squeak_admin_pb2.GetPeerRequest(
            peer_id=peer_id,
        )
    )
    assert not get_peer_response.HasField("squeak_peer")


def test_send_coins(admin_stub, other_admin_stub):
    new_address_response = admin_stub.LndNewAddress(ln.NewAddressRequest())
    get_balance_response = get_balance(
        other_admin_stub,
    )
    print("Get balance: {}".format(get_balance_response))
    print("--------------------")
    print("--------------------")
    print("--------------------")
    print("--------------------")
    print("--------------------")
    print("--------------------")
    print("--------------------")
    print("--------------------", flush=True)
    send_coins_response = send_coins(
        other_admin_stub, new_address_response.address, 55555555
    )
    time.sleep(10)
    get_transactions_response = admin_stub.LndGetTransactions(
        ln.GetTransactionsRequest()
    )

    assert any(
        [
            transaction.tx_hash == send_coins_response.txid
            for transaction in get_transactions_response.transactions
        ]
    )


def test_buy_squeak(
    admin_stub,
    other_admin_stub,
    signing_profile_id,
    saved_squeak_hash,
    admin_peer,
):
    # Download squeak
    download_squeak(other_admin_stub, saved_squeak_hash)
    # Download secret key
    download_squeak_secret_key(other_admin_stub, saved_squeak_hash)

    # Get the sent offers from the seller node
    get_sent_offers_response = admin_stub.GetSentOffers(
        squeak_admin_pb2.GetSentOffersRequest(),
    )
    squeak_hashes = [
        sent_offer.squeak_hash
        for sent_offer in get_sent_offers_response.sent_offers
    ]
    assert saved_squeak_hash in squeak_hashes

    # Get the buy offer
    get_buy_offers_response = other_admin_stub.GetBuyOffers(
        squeak_admin_pb2.GetBuyOffersRequest(
            squeak_hash=saved_squeak_hash,
        )
    )
    # print(get_buy_offers_response)
    assert len(get_buy_offers_response.offers) > 0

    offer = get_buy_offers_response.offers[0]

    print("Tring to connect LND peer with offer: {}".format(offer))
    with peer_connection(
        other_admin_stub, offer.node_host, offer.node_pubkey
    ), channel(other_admin_stub, offer.node_pubkey, 1000000):

        print("Channel context manager opened.")

        # Pay the offer
        list_channels_response = other_admin_stub.LndListChannels(
            ln.ListChannelsRequest(
                active_only=True,
            )
        )
        print("list_channels_response: {}".format(list_channels_response))

        # Pay the offer
        pay_offer_response = other_admin_stub.PayOffer(
            squeak_admin_pb2.PayOfferRequest(
                offer_id=offer.offer_id,
            )
        )
        # print(pay_offer_response)
        assert pay_offer_response.sent_payment_id > 0

        # Get the squeak display item
        get_squeak_display_entry = get_squeak_display(
            other_admin_stub, saved_squeak_hash)
        assert (
            get_squeak_display_entry.content_str == "Hello from the profile on the server!"
        )

        # Get all sent payments
        get_sent_payments_response = other_admin_stub.GetSentPayments(
            squeak_admin_pb2.GetSentPaymentsRequest(
                limit=10,
            ),
        )
        squeak_hashes = [
            sent_payment.squeak_hash
            for sent_payment in get_sent_payments_response.sent_payments
        ]
        assert saved_squeak_hash in squeak_hashes

        # Get the single sent payment
        for sent_payment in get_sent_payments_response.sent_payments:
            if sent_payment.squeak_hash == saved_squeak_hash:
                sent_payment_id = sent_payment.sent_payment_id
        get_sent_payment_response = other_admin_stub.GetSentPayment(
            squeak_admin_pb2.GetSentPaymentRequest(
                sent_payment_id=sent_payment_id,
            ),
        )
        assert saved_squeak_hash == get_sent_payment_response.sent_payment.squeak_hash
        assert get_sent_payment_response.sent_payment.price_msat == 1000000
        assert get_sent_payment_response.sent_payment.valid

        # Get the received payment from the seller node
        get_received_payments_response = admin_stub.GetReceivedPayments(
            squeak_admin_pb2.GetReceivedPaymentsRequest(
                limit=100,
            ),
        )
        # print(
        #     "get_received_payments_response: {}".format(
        #         get_received_payments_response)
        # )
        payment_hashes = [
            received_payment.payment_hash
            for received_payment in get_received_payments_response.received_payments
        ]
        assert sent_payment.payment_hash in payment_hashes
        for received_payment in get_received_payments_response.received_payments:
            received_payment_time_ms = received_payment.time_ms
            # print("received_payment_time_s: {}".format(
            #     received_payment_time_s))
            received_payment_time = datetime.datetime.fromtimestamp(
                received_payment_time_ms / 1000,
            )
            five_minutes = datetime.timedelta(minutes=5)
            assert received_payment_time > datetime.datetime.now() - five_minutes
            assert received_payment_time < datetime.datetime.now()
            assert len(received_payment.peer_address.host) > 4

        # Subscribe to received payments starting from index zero
        subscribe_received_payments_response = admin_stub.SubscribeReceivedPayments(
            squeak_admin_pb2.SubscribeReceivedPaymentsRequest(
                payment_index=0,
            ),
        )
        for payment in subscribe_received_payments_response:
            # print("Got payment from subscription: {}".format(payment))
            assert payment.received_payment_id == 1
            break

        # Get the payment summary from the seller node
        get_payment_summary_response = admin_stub.GetPaymentSummary(
            squeak_admin_pb2.GetPaymentSummaryRequest(),
        )
        # print(
        #     "get_payment_summary_response from seller: {}".format(
        #         get_payment_summary_response)
        # )
        assert get_payment_summary_response.payment_summary.num_received_payments > 0
        assert get_payment_summary_response.payment_summary.amount_earned_msat > 0

        # Get the payment summary from the buyer node
        get_payment_summary_response = other_admin_stub.GetPaymentSummary(
            squeak_admin_pb2.GetPaymentSummaryRequest(),
        )
        # print(
        #     "get_payment_summary_response from buyer: {}".format(
        #         get_payment_summary_response)
        # )
        assert get_payment_summary_response.payment_summary.num_sent_payments > 0
        assert get_payment_summary_response.payment_summary.amount_spent_msat > 0

    print("Channel context manager closed.")


def test_download_free_squeak(
    admin_stub,
    other_admin_stub,
    saved_squeak_hash,
    admin_peer,
):
    with free_price(admin_stub):
        # Download squeak
        download_result = download_squeak(other_admin_stub, saved_squeak_hash)
        # Download secret key
        download_squeak_secret_key(other_admin_stub, saved_squeak_hash)
        print('download_result:')
        print(download_result)
        assert download_result.number_downloaded == 1
        assert download_result.number_requested == 1

    # Get the squeak display item
    get_squeak_display_entry = get_squeak_display(
        other_admin_stub, saved_squeak_hash)
    assert (
        get_squeak_display_entry.content_str == "Hello from the profile on the server!"
    )


def test_download_single_squeak(
    admin_stub,
    other_admin_stub,
    signing_profile_id,
    saved_squeak_hash,
    admin_peer,
):

    with subscribe_squeak_entry(other_admin_stub, saved_squeak_hash) as subscription_queue, \
            subscribe_squeak_ancestor_entries(other_admin_stub, saved_squeak_hash) as ancestor_subscription_queue:

        # Get the squeak display item (should be empty)
        squeak_display_entry = get_squeak_display(
            other_admin_stub, saved_squeak_hash)
        assert squeak_display_entry is None

        # Get buy offers for the squeak hash (should be empty)
        get_buy_offers_response = other_admin_stub.GetBuyOffers(
            squeak_admin_pb2.GetBuyOffersRequest(
                squeak_hash=saved_squeak_hash,
            )
        )
        # print(get_buy_offers_response)
        assert len(get_buy_offers_response.offers) == 0

        # Download squeak
        download_result = download_squeak(other_admin_stub, saved_squeak_hash)
        # Download secret key
        download_squeak_secret_key(other_admin_stub, saved_squeak_hash)
        assert download_result.number_downloaded == 1
        assert download_result.number_requested == 1

        # Get the squeak display item
        squeak_display_entry = get_squeak_display(
            other_admin_stub, saved_squeak_hash)
        assert squeak_display_entry is not None

        # Get the buy offer
        get_buy_offers_response = other_admin_stub.GetBuyOffers(
            squeak_admin_pb2.GetBuyOffersRequest(
                squeak_hash=saved_squeak_hash,
            )
        )
        # print(get_buy_offers_response)
        assert len(get_buy_offers_response.offers) > 0

        item = subscription_queue.get()
        print("subscription_queue item:")
        print(item)
        assert item.squeak_hash == saved_squeak_hash

        item = ancestor_subscription_queue.get()
        print("ancestor_subscription_queue item:")
        print(item)
        assert item[0].squeak_hash == saved_squeak_hash


def test_like_squeak(admin_stub, saved_squeak_hash):
    # Get the squeak display item
    get_squeak_display_entry = get_squeak_display(
        admin_stub, saved_squeak_hash)
    assert (
        get_squeak_display_entry.liked_time_ms == 0
    )

    # Like the squeak
    admin_stub.LikeSqueak(
        squeak_admin_pb2.LikeSqueakRequest(
            squeak_hash=saved_squeak_hash,
        )
    )

    # Get the squeak display item
    get_squeak_display_entry = get_squeak_display(
        admin_stub, saved_squeak_hash)
    assert (
        get_squeak_display_entry.liked_time_ms > 0
    )

    # Unlike the squeak
    admin_stub.UnlikeSqueak(
        squeak_admin_pb2.UnlikeSqueakRequest(
            squeak_hash=saved_squeak_hash,
        )
    )

    # Get the squeak display item
    get_squeak_display_entry = get_squeak_display(
        admin_stub, saved_squeak_hash)
    assert (
        get_squeak_display_entry.liked_time_ms == 0
    )


def test_search(admin_stub, signing_profile_id):
    make_squeak_content = "Just some random weird text."
    make_squeak_hash = make_squeak(
        admin_stub, signing_profile_id, make_squeak_content)
    assert len(make_squeak_hash) == 32 * 2

    # Get all squeak displays for the given search text.
    search_results = get_search_squeaks(
        admin_stub,
        "Weird",
    )
    assert len(search_results) == 1

    # Get all squeak displays for other search text that shouldn't be there.
    missing_search_results = get_search_squeaks(
        admin_stub,
        "strange",
    )
    assert len(missing_search_results) == 0
