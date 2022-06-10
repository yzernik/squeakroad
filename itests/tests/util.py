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

import base64
import queue
import threading
import time
from contextlib import contextmanager

from squeak.core.elliptic import scalar_difference
from squeak.core.elliptic import scalar_from_bytes
from squeak.core.elliptic import scalar_to_bytes
from squeak.core.keys import SqueakPrivateKey
from squeak.core.keys import SqueakPublicKey

from proto import lnd_pb2
from proto import squeak_admin_pb2


def generate_private_key():
    return SqueakPrivateKey.generate()


def get_public_key(private_key):
    return private_key.get_public_key()


def public_key_from_hex(pubkey_hex):
    return SqueakPublicKey.from_bytes(bytes.fromhex(pubkey_hex))


def get_hash(squeak):
    """ Needs to be reversed because hash is stored as little-endian """
    hash_bytes = squeak.GetHash()[::-1]
    return hash_bytes.hex()


def string_to_hex(s):
    return bytes.fromhex(s)


def subtract_tweak(n, tweak):
    n_int = scalar_from_bytes(n)
    tweak_int = scalar_from_bytes(tweak)
    sum_int = scalar_difference(n_int, tweak_int)
    return scalar_to_bytes(sum_int)


def bytes_to_base64_string(data: bytes) -> str:
    encoded_string = base64.b64encode(data)
    return encoded_string.decode('utf-8')


@contextmanager
def peer_connection(node_stub, lightning_host, remote_pubkey):
    # Connect the peer
    connect_peer(node_stub, remote_pubkey, lightning_host)
    try:
        print("Yielding lnd peer is connected.")
        yield
    finally:
        # Disconnect the peer
        disconnect_peer(node_stub, remote_pubkey)
        time.sleep(2)


@contextmanager
def saved_peer(node_stub, peer_name, host, port):
    # Create a new peer
    peer_id = create_saved_peer(
        node_stub,
        peer_name,
        host,
        port,
    )
    yield peer_id
    # Delete the peer
    node_stub.DeletePeer(
        squeak_admin_pb2.DeletePeerRequest(
            peer_id=peer_id,
        )
    )


@contextmanager
def free_price(node_stub):
    # Set the price to zero
    set_sell_price(node_stub, 0)
    try:
        print("Yielding lnd peer is connected.")
        yield
    finally:
        # Clear the price
        clear_sell_price(node_stub)


@contextmanager
def channel(node_stub, remote_pubkey, amount):
    # Open channel to the server lightning node
    # pubkey_bytes = string_to_hex(remote_pubkey)
    # open_channel_response = lightning_client.open_channel(pubkey_bytes, amount)
    print("Trying to open channel...")
    channel_point = open_channel(node_stub, remote_pubkey, amount)
    print("Opening channel...")

    # Wait for channel to be open.
    MAX_RETRIES = 30
    i = 0
    while True:
        print("Try number: {}".format(i))
        # peers_list = lightning_client.list_peers()
        peers_list = list_peers(node_stub)
        # channels_list = lightning_client.list_channels()
        channels_list = list_channels(node_stub)
        # pending_channels_list = lightning_client.pending_channels()
        pending_channels_list = pending_channels(node_stub)
        print("list peers: {}".format(peers_list))
        print("list channels: {}".format(channels_list))
        print("pending channels: {}".format(pending_channels_list))
        if len(channels_list.channels) > 0:
            print("Channel now open.")
            break
        time.sleep(1)
        i += 1
        if i > MAX_RETRIES:
            raise Exception("Open channel timed out.")

    # for update in open_channel_response:
    #     if update.HasField("chan_open"):
    #         channel_point = update.chan_open.channel_point
    #         print("Channel now open: " + str(channel_point))
    #         break
    # print("list peers: {}".format(lightning_client.list_peers()))
    # print("list channels: {}".format(lightning_client.list_channels()))
    # print("pending channels: {}".format(lightning_client.pending_channels()))
    time.sleep(10)
    try:
        yield
    finally:
        # Code to release resource, e.g.:
        # Close the channel
        time.sleep(2)
        # for update in lightning_client.close_channel(channel_point):
        close_channel(node_stub, channel_point)


def create_saved_peer(node_stub, name, host, port):
    create_peer_response = node_stub.CreatePeer(
        squeak_admin_pb2.CreatePeerRequest(
            peer_name=name,
            peer_address=squeak_admin_pb2.PeerAddress(
                network="IPV4",
                host=host,
                port=port,
            )
        )
    )
    return create_peer_response.peer_id


def get_peer_by_address(node_stub, host, port):
    get_peer_by_address_response = node_stub.GetPeerByAddress(
        squeak_admin_pb2.GetPeerByAddressRequest(
            peer_address=squeak_admin_pb2.PeerAddress(
                network="IPV4",
                host="fake_host",
                port=1234,
            )
        )
    )
    return get_peer_by_address_response.squeak_peer


def get_search_squeaks(node_stub, search_text):
    get_search_squeak_display_response = node_stub.GetSearchSqueakDisplays(
        squeak_admin_pb2.GetSearchSqueakDisplaysRequest(
            search_text=search_text,
            limit=100,
        ),
    )
    return get_search_squeak_display_response.squeak_display_entries


def get_squeak_display(node_stub, squeak_hash):
    get_squeak_display_response = node_stub.GetSqueakDisplay(
        squeak_admin_pb2.GetSqueakDisplayRequest(
            squeak_hash=squeak_hash,
        )
    )
    if not get_squeak_display_response.HasField("squeak_display_entry"):
        return None
    return get_squeak_display_response.squeak_display_entry


def download_squeak(node_stub, squeak_hash):
    download_squeak_response = node_stub.DownloadSqueak(
        squeak_admin_pb2.DownloadSqueakRequest(
            squeak_hash=squeak_hash,
        ),
    )
    return download_squeak_response.download_result


def download_squeak_secret_key(node_stub, squeak_hash):
    download_squeak_secret_key_response = node_stub.DownloadSqueakSecretKey(
        squeak_admin_pb2.DownloadSqueakSecretKeyRequest(
            squeak_hash=squeak_hash,
        ),
    )
    return download_squeak_secret_key_response.download_result


def download_squeaks(node_stub, pubkeys_in_hex, min_block, max_block, reply_to):
    download_squeaks_response = node_stub.DownloadSqueaks(
        squeak_admin_pb2.DownloadSqueaksRequest(
            pubkeys=pubkeys_in_hex,
            min_block_height=min_block,
            max_block_height=max_block,
            replyto_squeak_hash=reply_to,
        ),
    )
    return download_squeaks_response.download_result


def download_squeaks_for_pubkey(node_stub, pubkey_hex):
    download_squeaks_response = node_stub.DownloadPubKeySqueaks(
        squeak_admin_pb2.DownloadPubKeySqueaksRequest(
            pubkey=pubkey_hex,
        ),
    )
    return download_squeaks_response.download_result


def download_offers(node_stub, squeak_hash):
    node_stub.DownloadOffers(
        squeak_admin_pb2.DownloadOffersRequest(
            squeak_hash=squeak_hash,
        ),
    )


def get_squeak_profile(node_stub, profile_id):
    get_squeak_profile_response = node_stub.GetSqueakProfile(
        squeak_admin_pb2.GetSqueakProfileRequest(
            profile_id=profile_id,
        )
    )
    if not get_squeak_profile_response.HasField("squeak_profile"):
        return None
    return get_squeak_profile_response.squeak_profile


def get_network(node_stub):
    get_network_response = node_stub.GetNetwork(
        squeak_admin_pb2.GetNetworkRequest()
    )
    return get_network_response.network


def set_sell_price(node_stub, price_msat):
    node_stub.SetSellPrice(
        squeak_admin_pb2.SetSellPriceRequest(
            price_msat=price_msat,
        )
    )


def clear_sell_price(node_stub):
    node_stub.ClearSellPrice(
        squeak_admin_pb2.ClearSellPriceRequest()
    )


def get_sell_price(node_stub):
    get_sell_price_response = node_stub.GetSellPrice(
        squeak_admin_pb2.GetSellPriceRequest()
    )
    return get_sell_price_response


def get_external_address(node_stub):
    get_external_address_response = node_stub.GetExternalAddress(
        squeak_admin_pb2.GetExternalAddressRequest()
    )
    return get_external_address_response.peer_address


def make_squeak(node_stub, profile_id, squeak_content, reply_to_hash=None, recipient_profile_id=None):
    make_squeak_response = node_stub.MakeSqueak(
        squeak_admin_pb2.MakeSqueakRequest(
            profile_id=profile_id,
            content=squeak_content,
            replyto=reply_to_hash,
            has_recipient=(recipient_profile_id is not None),
            recipient_profile_id=recipient_profile_id,
        )
    )
    return make_squeak_response.squeak_hash


def delete_squeak(node_stub, squeak_hash):
    node_stub.DeleteSqueak(
        squeak_admin_pb2.DeleteSqueakRequest(squeak_hash=squeak_hash)
    )


def create_contact_profile(node_stub, profile_name, public_key):
    create_contact_profile_response = node_stub.CreateContactProfile(
        squeak_admin_pb2.CreateContactProfileRequest(
            profile_name=profile_name,
            pubkey=public_key.to_bytes().hex(),
        )
    )
    return create_contact_profile_response.profile_id


def create_signing_profile(node_stub, profile_name):
    create_signing_profile_response = node_stub.CreateSigningProfile(
        squeak_admin_pb2.CreateSigningProfileRequest(
            profile_name=profile_name,
        )
    )
    return create_signing_profile_response.profile_id


def get_profile_by_pubkey(node_stub, pubkey):
    get_squeak_profile_by_pubkey_response = node_stub.GetSqueakProfileByPubKey(
        squeak_admin_pb2.GetSqueakProfileByPubKeyRequest(
            pubkey=pubkey,
        )
    )
    if not get_squeak_profile_by_pubkey_response.HasField("squeak_profile"):
        return None
    return get_squeak_profile_by_pubkey_response.squeak_profile


def get_pubkey_squeak_displays(node_stub, pubkey, limit):
    get_pubkey_squeak_display_response = node_stub.GetPubKeySqueakDisplays(
        squeak_admin_pb2.GetPubKeySqueakDisplaysRequest(
            pubkey=pubkey,
            limit=limit,
        ),
    )
    return get_pubkey_squeak_display_response.squeak_display_entries


def import_signing_profile(node_stub, profile_name, private_key):
    import_response = node_stub.ImportSigningProfile(
        squeak_admin_pb2.ImportSigningProfileRequest(
            profile_name=profile_name,
            private_key=private_key,
        )
    )
    return import_response.profile_id


def delete_profile(node_stub, profile_id):
    node_stub.DeleteSqueakProfile(
        squeak_admin_pb2.DeleteSqueakProfileRequest(
            profile_id=profile_id,
        )
    )


def open_channel(node_stub, remote_pubkey, amount):
    return node_stub.LndOpenChannelSync(
        lnd_pb2.OpenChannelRequest(
            node_pubkey_string=remote_pubkey,
            local_funding_amount=amount,
        )
    )


def close_channel(node_stub, channel_point):
    return node_stub.LndCloseChannel(
        lnd_pb2.CloseChannelRequest(
            channel_point=channel_point,
        )
    )


def list_peers(node_stub):
    return node_stub.LndListPeers(
        lnd_pb2.ListPeersRequest()
    )


def list_channels(node_stub):
    return node_stub.LndListChannels(
        lnd_pb2.ListChannelsRequest()
    )


def pending_channels(node_stub):
    return node_stub.LndPendingChannels(
        lnd_pb2.PendingChannelsRequest()
    )


def connect_peer(node_stub, pubkey, host):
    lightning_address = lnd_pb2.LightningAddress(
        pubkey=pubkey,
        host=host,
    )
    connect_peer_request = lnd_pb2.ConnectPeerRequest(
        addr=lightning_address,
    )
    return node_stub.LndConnectPeer(connect_peer_request)


def disconnect_peer(node_stub, pubkey):
    disconnect_peer_request = lnd_pb2.DisconnectPeerRequest(
        pub_key=pubkey,
    )
    return node_stub.LndDisconnectPeer(
        disconnect_peer_request,
    )


def get_balance(node_stub):
    get_balance_request = lnd_pb2.WalletBalanceRequest()
    return node_stub.LndWalletBalance(
        get_balance_request,
    )


def send_coins(node_stub, addr, amount):
    send_coins_request = lnd_pb2.SendCoinsRequest(
        addr=addr,
        amount=amount,
    )
    return node_stub.LndSendCoins(
        send_coins_request,
    )


@contextmanager
def subscribe_squeak_entry(node_stub, squeak_hash):
    q = queue.Queue()
    subscribe_squeak_entry_response = node_stub.SubscribeSqueakDisplay(
        squeak_admin_pb2.SubscribeSqueakDisplayRequest(
            squeak_hash=squeak_hash,
        )
    )

    def enqueue_results():
        for result in subscribe_squeak_entry_response:
            q.put(result.squeak_display_entry)

    threading.Thread(
        target=enqueue_results,
    ).start()
    yield q
    subscribe_squeak_entry_response.cancel()


@contextmanager
def subscribe_squeaks_for_address(node_stub, pubkey_hex):
    q = queue.Queue()
    subscribe_address_squeaks_response = node_stub.SubscribePubKeySqueakDisplays(
        squeak_admin_pb2.SubscribePubKeySqueakDisplaysRequest(
            pubkey=pubkey_hex,
        )
    )

    def enqueue_results():
        for result in subscribe_address_squeaks_response:
            q.put(result.squeak_display_entry)

    threading.Thread(
        target=enqueue_results,
    ).start()
    yield q
    subscribe_address_squeaks_response.cancel()


@contextmanager
def subscribe_squeak_ancestor_entries(node_stub, squeak_hash):
    q = queue.Queue()
    subscribe_squeak_ancestor_entries_response = node_stub.SubscribeAncestorSqueakDisplays(
        squeak_admin_pb2.SubscribeAncestorSqueakDisplaysRequest(
            squeak_hash=squeak_hash,
        )
    )

    def enqueue_results():
        for result in subscribe_squeak_ancestor_entries_response:
            q.put(result.squeak_display_entries)

    threading.Thread(
        target=enqueue_results,
    ).start()
    yield q
    subscribe_squeak_ancestor_entries_response.cancel()
