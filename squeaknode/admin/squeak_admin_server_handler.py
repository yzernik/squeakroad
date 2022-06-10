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
import logging

from squeak.core.keys import SqueakPrivateKey
from squeak.core.keys import SqueakPublicKey

from proto import squeak_admin_pb2
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
from squeaknode.admin.messages import twitter_account_to_message
from squeaknode.admin.profile_image_util import base64_string_to_bytes
from squeaknode.lightning.lnd_lightning_client import LNDLightningClient
from squeaknode.node.squeak_controller import SqueakController

logger = logging.getLogger(__name__)


class SqueakAdminServerHandler(object):
    """Handles admin server commands."""

    def __init__(
        self,
        lightning_client: LNDLightningClient,
        squeak_controller: SqueakController,
    ):
        self.lightning_client = lightning_client
        self.squeak_controller = squeak_controller

    def handle_lnd_get_info(self, request):
        logger.info("Handle lnd get info")
        return self.lightning_client.stub.GetInfo(request)

    def handle_lnd_wallet_balance(self, request):
        logger.info("Handle lnd wallet balance")
        return self.lightning_client.stub.WalletBalance(request)

    def handle_lnd_new_address(self, request):
        logger.info("Handle lnd new address: {}".format(request))
        return self.lightning_client.stub.NewAddress(request)

    def handle_lnd_list_channels(self, request):
        logger.info("Handle lnd list channels")
        return self.lightning_client.stub.ListChannels(request)

    def handle_lnd_pending_channels(self, request):
        logger.info("Handle lnd pending channels")
        return self.lightning_client.stub.PendingChannels(request)

    def handle_lnd_get_transactions(self, request):
        logger.info("Handle lnd get transactions")
        return self.lightning_client.stub.GetTransactions(request)

    def handle_lnd_list_peers(self, request):
        logger.info("Handle list peers")
        return self.lightning_client.stub.ListPeers(request)

    def handle_lnd_connect_peer(self, request):
        logger.info("Handle connect peer: {}".format(request))
        return self.lightning_client.stub.ConnectPeer(request)

    def handle_lnd_disconnect_peer(self, request):
        logger.info("Handle disconnect peer: {}".format(request))
        return self.lightning_client.stub.DisconnectPeer(request)

    def handle_lnd_open_channel_sync(self, request):
        logger.info("Handle open channel: {}".format(request))
        return self.lightning_client.stub.OpenChannelSync(request)

    def handle_lnd_close_channel(self, request):
        logger.info("Handle close channel: {}".format(request))
        return self.lightning_client.stub.CloseChannel(request)

    def handle_lnd_subscribe_channel_events(self, request):
        logger.info("Handle subscribe channel events")
        return self.lightning_client.stub.SubscribeChannelEvents(request)

    def handle_lnd_send_coins(self, request):
        logger.info("Handle send coins.")
        return self.lightning_client.stub.SendCoins(request)

    def handle_create_signing_profile(self, request):
        profile_name = request.profile_name
        logger.info(
            "Handle create signing profile with name: {}".format(profile_name))
        profile_id = self.squeak_controller.create_signing_profile(
            profile_name)
        logger.info("New profile_id: {}".format(profile_id))
        return squeak_admin_pb2.CreateSigningProfileReply(
            profile_id=profile_id,
        )

    def handle_import_signing_profile(self, request):
        profile_name = request.profile_name
        private_key_hex = request.private_key
        logger.info(
            "Handle import signing profile with name: {}".format(profile_name))
        private_key = SqueakPrivateKey.from_bytes(
            bytes.fromhex(private_key_hex))
        profile_id = self.squeak_controller.import_signing_profile(
            profile_name, private_key)
        logger.info("New profile_id: {}".format(profile_id))
        return squeak_admin_pb2.ImportSigningProfileReply(
            profile_id=profile_id,
        )

    def handle_create_contact_profile(self, request):
        profile_name = request.profile_name
        public_key_hex = request.pubkey
        logger.info(
            "Handle create contact profile with name: {}, public key: {}".format(
                profile_name,
                public_key_hex,
            )
        )
        public_key = SqueakPublicKey.from_bytes(bytes.fromhex(public_key_hex))
        profile_id = self.squeak_controller.create_contact_profile(
            profile_name,
            public_key,
        )
        logger.info("New profile_id: {}".format(profile_id))
        return squeak_admin_pb2.CreateContactProfileReply(
            profile_id=profile_id,
        )

    def handle_get_profiles(self, request):
        logger.info("Handle get profiles.")
        profiles = self.squeak_controller.get_profiles()
        logger.info("Got number of profiles: {}".format(len(profiles)))
        profile_msgs = [squeak_profile_to_message(
            profile) for profile in profiles]
        return squeak_admin_pb2.GetProfilesReply(squeak_profiles=profile_msgs)

    def handle_get_signing_profiles(self, request):
        logger.info("Handle get signing profiles.")
        profiles = self.squeak_controller.get_signing_profiles()
        logger.info("Got number of signing profiles: {}".format(len(profiles)))
        profile_msgs = [squeak_profile_to_message(
            profile) for profile in profiles]
        return squeak_admin_pb2.GetSigningProfilesReply(squeak_profiles=profile_msgs)

    def handle_get_contact_profiles(self, request):
        logger.info("Handle get contact profiles.")
        profiles = self.squeak_controller.get_contact_profiles()
        logger.info("Got number of contact profiles: {}".format(len(profiles)))
        profile_msgs = [squeak_profile_to_message(
            profile) for profile in profiles]
        return squeak_admin_pb2.GetContactProfilesReply(squeak_profiles=profile_msgs)

    def handle_get_squeak_profile(self, request):
        profile_id = request.profile_id
        logger.info("Handle get squeak profile with id: {}".format(profile_id))
        squeak_profile = self.squeak_controller.get_squeak_profile(profile_id)
        squeak_profile_msg = optional_squeak_profile_to_message(squeak_profile)
        return squeak_admin_pb2.GetSqueakProfileReply(
            squeak_profile=squeak_profile_msg,
        )

    def handle_get_squeak_profile_by_pubkey(self, request):
        public_key_hex = request.pubkey
        logger.info(
            "Handle get squeak profile with public key: {}".format(public_key_hex))
        public_key = SqueakPublicKey.from_bytes(bytes.fromhex(public_key_hex))
        squeak_profile = self.squeak_controller.get_squeak_profile_by_public_key(
            public_key,
        )
        squeak_profile_msg = optional_squeak_profile_to_message(squeak_profile)
        return squeak_admin_pb2.GetSqueakProfileByPubKeyReply(
            squeak_profile=squeak_profile_msg
        )

    def handle_get_squeak_profile_by_name(self, request):
        name = request.name
        logger.info("Handle get squeak profile with name: {}".format(name))
        squeak_profile = self.squeak_controller.get_squeak_profile_by_name(
            name)
        squeak_profile_msg = optional_squeak_profile_to_message(squeak_profile)
        return squeak_admin_pb2.GetSqueakProfileByNameReply(
            squeak_profile=squeak_profile_msg
        )

    def handle_set_squeak_profile_following(self, request):
        profile_id = request.profile_id
        following = request.following
        logger.info(
            "Handle set squeak profile following with profile id: {}, following: {}".format(
                profile_id,
                following,
            )
        )
        self.squeak_controller.set_squeak_profile_following(
            profile_id, following)
        return squeak_admin_pb2.SetSqueakProfileFollowingReply()

    def handle_rename_squeak_profile(self, request):
        profile_id = request.profile_id
        profile_name = request.profile_name
        logger.info(
            "Handle rename squeak profile with profile id: {}, new name: {}".format(
                profile_id,
                profile_name,
            )
        )
        self.squeak_controller.rename_squeak_profile(profile_id, profile_name)
        return squeak_admin_pb2.RenameSqueakProfileReply()

    def handle_delete_squeak_profile(self, request):
        profile_id = request.profile_id
        logger.info(
            "Handle delete squeak profile with id: {}".format(profile_id))
        self.squeak_controller.delete_squeak_profile(profile_id)
        return squeak_admin_pb2.DeleteSqueakProfileReply()

    def handle_set_squeak_profile_image(self, request):
        profile_id = request.profile_id
        profile_image = request.profile_image
        logger.info(
            "Handle set squeak profile image with profile id: {}".format(
                profile_id,
            )
        )
        profile_image_bytes = base64_string_to_bytes(profile_image)
        self.squeak_controller.set_squeak_profile_image(
            profile_id, profile_image_bytes)
        return squeak_admin_pb2.SetSqueakProfileImageReply()

    def handle_clear_squeak_profile_image(self, request):
        profile_id = request.profile_id
        logger.info(
            "Handle clear squeak profile image with profile id: {}".format(
                profile_id,
            )
        )
        self.squeak_controller.clear_squeak_profile_image(
            profile_id,
        )
        return squeak_admin_pb2.ClearSqueakProfileImageReply()

    def handle_get_squeak_profile_private_key(self, request):
        profile_id = request.profile_id
        logger.info(
            "Handle get squeak profile private key for id: {}".format(profile_id))
        private_key = self.squeak_controller.get_squeak_profile_private_key(
            profile_id)
        private_key_hex = private_key.to_bytes().hex()
        return squeak_admin_pb2.GetSqueakProfilePrivateKeyReply(
            private_key=private_key_hex,
        )

    def handle_make_squeak(self, request):
        profile_id = request.profile_id
        content_str = request.content
        replyto_hash_str = request.replyto
        replyto_hash = bytes.fromhex(
            replyto_hash_str) if replyto_hash_str else None
        has_recipient = request.has_recipient
        recipient_profile_id = request.recipient_profile_id if has_recipient else None
        logger.info(
            "Handle make squeak with author profile id: {}, and recipient profile id: {}".format(
                profile_id, recipient_profile_id)
        )
        inserted_squeak_hash = self.squeak_controller.make_squeak(
            profile_id,
            content_str,
            replyto_hash,
            recipient_profile_id,
        )
        inserted_squeak_hash_str = optional_squeak_hash_to_hex(
            inserted_squeak_hash,
        )
        return squeak_admin_pb2.MakeSqueakReply(
            squeak_hash=inserted_squeak_hash_str,
        )

    def handle_make_resqueak(self, request):
        profile_id = request.profile_id
        resqueaked_hash_str = request.resqueaked_hash
        resqueaked_hash = bytes.fromhex(resqueaked_hash_str)
        replyto_hash_str = request.replyto
        replyto_hash = bytes.fromhex(
            replyto_hash_str) if replyto_hash_str else None
        logger.info(
            "Handle make resqueak with author profile id: {}".format(
                profile_id,
            )
        )
        inserted_resqueak_hash = self.squeak_controller.make_resqueak(
            profile_id,
            resqueaked_hash,
            replyto_hash,
        )
        inserted_resqueak_hash_str = optional_squeak_hash_to_hex(
            inserted_resqueak_hash,
        )
        return squeak_admin_pb2.MakeResqueakReply(
            squeak_hash=inserted_resqueak_hash_str,
        )

    def handle_get_squeak_display_entry(self, request):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle get squeak display entry for hash: {}".format(squeak_hash_str))
        squeak_entry = (
            self.squeak_controller.get_squeak_entry(
                squeak_hash
            )
        )
        display_message = optional_squeak_entry_to_message(
            squeak_entry)
        return squeak_admin_pb2.GetSqueakDisplayReply(
            squeak_display_entry=display_message
        )

    def handle_get_timeline_squeak_display_entries(self, request):
        limit = request.limit
        last_entry = message_to_squeak_entry(request.last_entry) if request.HasField(
            "last_entry") else None
        logger.info("""Handle get timeline squeak display entries with
        limit: {}
        last_entry: {}
        """.format(
            limit,
            last_entry,
        ))
        squeak_entries = (
            self.squeak_controller.get_timeline_squeak_entries(
                limit,
                last_entry,
            )
        )
        logger.info(
            "Got number of timeline squeak entries: {}".format(
                len(squeak_entries)
            )
        )
        squeak_display_msgs = [
            squeak_entry_to_message(entry) for entry in squeak_entries
        ]
        return squeak_admin_pb2.GetTimelineSqueakDisplaysReply(
            squeak_display_entries=squeak_display_msgs
        )

    def handle_get_squeak_display_entries_for_pubkey(self, request):
        public_key_hex = request.pubkey
        limit = request.limit
        last_entry = message_to_squeak_entry(request.last_entry) if request.HasField(
            "last_entry") else None
        logger.info("""Handle get squeak display entries for public key: {} with
        limit: {}
        last_entry: {}
        """.format(
            public_key_hex,
            limit,
            last_entry,
        ))
        public_key = SqueakPublicKey.from_bytes(bytes.fromhex(public_key_hex))
        squeak_entries = (
            self.squeak_controller.get_squeak_entries_for_public_key(
                public_key,
                limit,
                last_entry,
            )
        )
        logger.info(
            "Got number of squeak entries for pubkey: {}".format(
                len(squeak_entries)
            )
        )
        squeak_display_msgs = [
            squeak_entry_to_message(entry) for entry in squeak_entries
        ]
        return squeak_admin_pb2.GetPubKeySqueakDisplaysReply(
            squeak_display_entries=squeak_display_msgs
        )

    def handle_get_squeak_display_entries_for_text_search(self, request):
        search_text = request.search_text
        limit = request.limit
        last_entry = message_to_squeak_entry(request.last_entry) if request.HasField(
            "last_entry") else None
        logger.info("""Handle get squeak display entries for search_text: {} with
        limit: {}
        last_entry: {}
        """.format(
            search_text,
            limit,
            last_entry,
        ))
        squeak_entries = (
            self.squeak_controller.get_squeak_entries_for_text_search(
                search_text,
                limit,
                last_entry,
            )
        )
        logger.info(
            "Got number of squeak entries for text search: {}".format(
                len(squeak_entries)
            )
        )
        squeak_display_msgs = [
            squeak_entry_to_message(entry) for entry in squeak_entries
        ]
        return squeak_admin_pb2.GetSearchSqueakDisplaysReply(
            squeak_display_entries=squeak_display_msgs
        )

    def handle_get_ancestor_squeak_display_entries(self, request):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle get ancestor squeak display entries for squeak hash: {}".format(
                squeak_hash_str
            )
        )
        squeak_entries = (
            self.squeak_controller.get_ancestor_squeak_entries(
                squeak_hash,
            )
        )
        logger.info(
            "Got number of ancestor squeak entries: {}".format(
                len(squeak_entries)
            )
        )
        squeak_display_msgs = [
            squeak_entry_to_message(entry) for entry in squeak_entries
        ]
        return squeak_admin_pb2.GetAncestorSqueakDisplaysReply(
            squeak_display_entries=squeak_display_msgs
        )

    def handle_get_reply_squeak_display_entries(self, request):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        limit = request.limit
        last_entry = message_to_squeak_entry(request.last_entry) if request.HasField(
            "last_entry") else None
        logger.info("""Handle get reply squeak display entries for squeak hash: {} with
        limit: {}
        last_entry: {}
        """.format(
            squeak_hash_str,
            limit,
            last_entry,
        ))
        squeak_entries = (
            self.squeak_controller.get_reply_squeak_entries(
                squeak_hash,
                limit,
                last_entry,
            )
        )
        logger.info(
            "Got number of reply squeak entries: {}".format(
                len(squeak_entries)
            )
        )
        squeak_display_msgs = [
            squeak_entry_to_message(entry) for entry in squeak_entries
        ]
        return squeak_admin_pb2.GetReplySqueakDisplaysReply(
            squeak_display_entries=squeak_display_msgs
        )

    def handle_delete_squeak(self, request):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle delete squeak with hash: {}".format(squeak_hash_str))
        self.squeak_controller.delete_squeak(squeak_hash)
        logger.info(
            "Deleted squeak entry with hash: {}".format(squeak_hash_str))
        return squeak_admin_pb2.DeleteSqueakReply()

    def handle_create_peer(self, request):
        peer_name = request.peer_name
        peer_address = message_to_peer_address(request.peer_address)
        logger.info(
            "Handle create peer with name: {}, address: {}".format(
                peer_name,
                peer_address,
            )
        )
        peer_id = self.squeak_controller.create_peer(
            peer_name,
            peer_address,
        )
        return squeak_admin_pb2.CreatePeerReply(
            peer_id=peer_id,
        )

    def handle_get_squeak_peer(self, request):
        peer_id = request.peer_id
        logger.info("Handle get squeak peer with id: {}".format(peer_id))
        squeak_peer = self.squeak_controller.get_peer(peer_id)
        logger.info("Got squeak peer: {}".format(squeak_peer))
        squeak_peer_msg = optional_squeak_peer_to_message(squeak_peer)
        return squeak_admin_pb2.GetPeerReply(
            squeak_peer=squeak_peer_msg,
        )

    def handle_get_squeak_peer_by_address(self, request):
        peer_address = message_to_peer_address(request.peer_address)
        logger.info(
            "Handle get squeak peer with address: {}".format(peer_address))
        squeak_peer = self.squeak_controller.get_peer_by_address(peer_address)
        squeak_peer_msg = optional_squeak_peer_to_message(squeak_peer)
        return squeak_admin_pb2.GetPeerByAddressReply(
            squeak_peer=squeak_peer_msg,
        )

    def handle_get_squeak_peers(self, request):
        logger.info("Handle get squeak peers")
        squeak_peers = self.squeak_controller.get_peers()
        squeak_peer_msgs = [
            squeak_peer_to_message(squeak_peer)
            for squeak_peer in squeak_peers
        ]
        return squeak_admin_pb2.GetPeersReply(
            squeak_peers=squeak_peer_msgs,
        )

    def handle_rename_squeak_peer(self, request):
        peer_id = request.peer_id
        peer_name = request.peer_name
        logger.info(
            "Handle rename peer with peer id: {}, new name: {}".format(
                peer_id,
                peer_name,
            )
        )
        self.squeak_controller.rename_peer(peer_id, peer_name)
        return squeak_admin_pb2.RenamePeerReply()

    def handle_set_squeak_peer_autoconnect(self, request):
        peer_id = request.peer_id
        autoconnect = request.autoconnect
        logger.info(
            "Handle set peer autoconnect with peer id: {}, autoconnect: {}".format(
                peer_id,
                autoconnect,
            )
        )
        self.squeak_controller.set_peer_autoconnect(peer_id, autoconnect)
        return squeak_admin_pb2.SetPeerAutoconnectReply()

    def handle_set_squeak_peer_share_for_free(self, request):
        peer_id = request.peer_id
        share_for_free = request.share_for_free
        logger.info(
            "Handle set peer share_for_free with peer id: {}, share_for_free: {}".format(
                peer_id,
                share_for_free,
            )
        )
        self.squeak_controller.set_peer_share_for_free(peer_id, share_for_free)
        return squeak_admin_pb2.SetPeerShareForFreeReply()

    def handle_delete_squeak_peer(self, request):
        peer_id = request.peer_id
        logger.info("Handle delete squeak peer with id: {}".format(peer_id))
        self.squeak_controller.delete_peer(peer_id)
        return squeak_admin_pb2.DeletePeerReply()

    def handle_get_buy_offers(self, request):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle get received offers for hash: {}".format(squeak_hash_str))
        offers = self.squeak_controller.get_received_offers(
            squeak_hash)
        offer_msgs = [received_offer_to_message(offer) for offer in offers]
        return squeak_admin_pb2.GetBuyOffersReply(
            offers=offer_msgs,
        )

    def handle_get_buy_offer(self, request):
        offer_id = request.offer_id
        logger.info("Handle get buy offer for hash: {}".format(offer_id))
        received_offer = self.squeak_controller.get_received_offer(offer_id)
        received_offer_msg = optional_received_offer_to_message(received_offer)
        return squeak_admin_pb2.GetBuyOfferReply(
            offer=received_offer_msg,
        )

    def handle_download_squeaks(self, request):
        pubkeys_in_hex = request.pubkeys
        min_block = request.min_block_height
        max_block = request.max_block_height
        replyto_hash = request.replyto_squeak_hash
        logger.info("""Handle download squeaks for
        public keys: {}
        min_block: {}
        max_block: {}
        replyto_hash: {}
        """.format(
            pubkeys_in_hex,
            min_block,
            max_block,
            replyto_hash,
        ))
        public_keys = [
            SqueakPublicKey.from_bytes(bytes.fromhex(pubkey_hex))
            for pubkey_hex in pubkeys_in_hex
        ]
        download_result = self.squeak_controller.download_squeaks(
            public_keys,
            min_block,
            max_block,
            replyto_hash,
        )
        logger.info("Download result: {}".format(download_result))
        download_result_msg = download_result_to_message(download_result)
        return squeak_admin_pb2.DownloadSqueaksReply(
            download_result=download_result_msg,
        )

    def handle_download_squeak(self, request):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle download squeak with hash: {}".format(squeak_hash_str))
        download_result = self.squeak_controller.download_single_squeak(
            squeak_hash)
        logger.info("Download result: {}".format(download_result))
        download_result_msg = download_result_to_message(download_result)
        return squeak_admin_pb2.DownloadSqueakReply(
            download_result=download_result_msg,
        )

    def handle_download_squeak_secret_key(self, request):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle download squeak secret key for hash: {}".format(squeak_hash_str))
        download_result = self.squeak_controller.download_single_squeak_secret_key(
            squeak_hash)
        logger.info("Download result: {}".format(download_result))
        download_result_msg = download_result_to_message(download_result)
        return squeak_admin_pb2.DownloadSqueakReply(
            download_result=download_result_msg,
        )

    def handle_download_offers(self, request):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle download offer for hash: {}".format(squeak_hash_str))
        download_result = self.squeak_controller.download_offers(squeak_hash)
        logger.info("Download result: {}".format(download_result))
        download_result_msg = download_result_to_message(download_result)
        return squeak_admin_pb2.DownloadOffersReply(
            download_result=download_result_msg,
        )

    def handle_download_replies(self, request):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle download replies for hash: {}".format(squeak_hash_str))
        download_result = self.squeak_controller.download_replies(squeak_hash)
        logger.info("Download result: {}".format(download_result))
        download_result_msg = download_result_to_message(download_result)
        return squeak_admin_pb2.DownloadRepliesReply(
            download_result=download_result_msg,
        )

    def handle_download_pubkey_squeaks(self, request):
        public_key_hex = request.pubkey
        logger.info(
            "Handle download squeaks for public key: {}".format(public_key_hex))
        public_key = SqueakPublicKey.from_bytes(bytes.fromhex(public_key_hex))
        download_result = self.squeak_controller.download_public_key_squeaks(
            public_key,
        )
        logger.info("Download result: {}".format(download_result))
        download_result_msg = download_result_to_message(download_result)
        return squeak_admin_pb2.DownloadPubKeySqueaksReply(
            download_result=download_result_msg,
        )

    def handle_pay_offer(self, request):
        offer_id = request.offer_id
        logger.info("Handle pay offer for offer id: {}".format(offer_id))
        sent_payment_id = self.squeak_controller.pay_offer(offer_id)
        return squeak_admin_pb2.PayOfferReply(
            sent_payment_id=sent_payment_id,
        )

    def handle_decrypt_squeak(self, request):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        has_author = request.has_author
        author_profile_id = request.author_profile_id if has_author else None
        has_recipient = request.has_recipient
        recipient_profile_id = request.recipient_profile_id if has_recipient else None
        logger.info(
            "Handle decrypt squeak with hash: {}, recipient id: {}, author id: {}".format(
                squeak_hash_str,
                recipient_profile_id,
                author_profile_id,
            ),
        )
        self.squeak_controller.decrypt_private_squeak(
            squeak_hash=squeak_hash,
            author_profile_id=author_profile_id,
            recipient_profile_id=recipient_profile_id,
        )
        return squeak_admin_pb2.DecryptSqueakReply()

    def handle_get_sent_payments(self, request):
        limit = request.limit
        last_sent_payment = message_to_sent_payment(request.last_sent_payment) if request.HasField(
            "last_sent_payment") else None
        logger.info("""Handle get sent payments with
        limit: {}
        last_sent_payment: {}
        """.format(
            limit,
            last_sent_payment,
        ))
        sent_payments = self.squeak_controller.get_sent_payments(
            limit,
            last_sent_payment,
        )
        logger.info(
            "Got number of sent payments: {}".format(
                len(sent_payments)
            )
        )
        sent_payment_msgs = [
            sent_payment_to_message(sent_payment)
            for sent_payment in sent_payments
        ]
        return squeak_admin_pb2.GetSentPaymentsReply(
            sent_payments=sent_payment_msgs,
        )

    def handle_get_sent_payments_for_squeak(self, request):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        limit = request.limit
        last_sent_payment = message_to_sent_payment(request.last_sent_payment) if request.HasField(
            "last_sent_payment") else None
        logger.info("""Handle get sent payments with
        squeak_hash: {}
        limit: {}
        last_sent_payment: {}
        """.format(
            squeak_hash_str,
            limit,
            last_sent_payment,
        ))
        sent_payments = self.squeak_controller.get_sent_payments_for_squeak(
            squeak_hash,
            limit,
            last_sent_payment,
        )
        logger.info(
            "Got number of sent payments: {}".format(
                len(sent_payments)
            )
        )
        sent_payment_msgs = [
            sent_payment_to_message(sent_payment)
            for sent_payment in sent_payments
        ]
        return squeak_admin_pb2.GetSentPaymentsReply(
            sent_payments=sent_payment_msgs,
        )

    def handle_get_sent_payments_for_pubkey(self, request):
        public_key_hex = request.pubkey
        limit = request.limit
        last_sent_payment = message_to_sent_payment(request.last_sent_payment) if request.HasField(
            "last_sent_payment") else None
        logger.info("""Handle get sent payments with
        public_key: {}
        limit: {}
        last_sent_payment: {}
        """.format(
            public_key_hex,
            limit,
            last_sent_payment,
        ))
        public_key = SqueakPublicKey.from_bytes(bytes.fromhex(public_key_hex))
        sent_payments = self.squeak_controller.get_sent_payments_for_pubkey(
            public_key,
            limit,
            last_sent_payment,
        )
        logger.info(
            "Got number of sent payments: {}".format(
                len(sent_payments)
            )
        )
        sent_payment_msgs = [
            sent_payment_to_message(sent_payment)
            for sent_payment in sent_payments
        ]
        return squeak_admin_pb2.GetSentPaymentsReply(
            sent_payments=sent_payment_msgs,
        )

    def handle_get_sent_payments_for_peer(self, request):
        peer_address = message_to_peer_address(request.peer_address)
        limit = request.limit
        last_sent_payment = message_to_sent_payment(request.last_sent_payment) if request.HasField(
            "last_sent_payment") else None
        logger.info("""Handle get sent payments with
        peer_address: {}
        limit: {}
        last_sent_payment: {}
        """.format(
            peer_address,
            limit,
            last_sent_payment,
        ))
        sent_payments = self.squeak_controller.get_sent_payments_for_peer(
            peer_address,
            limit,
            last_sent_payment,
        )
        logger.info(
            "Got number of sent payments: {}".format(
                len(sent_payments)
            )
        )
        sent_payment_msgs = [
            sent_payment_to_message(sent_payment)
            for sent_payment in sent_payments
        ]
        return squeak_admin_pb2.GetSentPaymentsReply(
            sent_payments=sent_payment_msgs,
        )

    def handle_get_sent_payment(self, request):
        sent_payment_id = request.sent_payment_id
        logger.info(
            "Handle get sent payment with id: {}".format(sent_payment_id))
        sent_payment = self.squeak_controller.get_sent_payment(sent_payment_id)
        sent_payment_msg = optional_sent_payment_to_message(sent_payment)
        return squeak_admin_pb2.GetSentPaymentReply(
            sent_payment=sent_payment_msg,
        )

    def handle_get_sent_offers(self, request):
        logger.info("Handle get sent offers")
        sent_offers = self.squeak_controller.get_sent_offers()
        sent_offer_msgs = [
            sent_offer_to_message(sent_offer) for sent_offer in sent_offers
        ]
        return squeak_admin_pb2.GetSentOffersReply(
            sent_offers=sent_offer_msgs,
        )

    def handle_get_received_payments(self, request):
        limit = request.limit
        last_received_payment = message_to_received_payment(request.last_received_payment) if request.HasField(
            "last_received_payment") else None
        logger.info("""Handle get received payments with
        limit: {}
        last_received_payment: {}
        """.format(
            limit,
            last_received_payment,
        ))
        received_payments = self.squeak_controller.get_received_payments(
            limit,
            last_received_payment,
        )
        logger.info(
            "Got number of received payments: {}".format(
                len(received_payments)
            )
        )
        received_payment_msgs = [
            received_payment_to_message(received_payment)
            for received_payment in received_payments
        ]
        return squeak_admin_pb2.GetReceivedPaymentsReply(
            received_payments=received_payment_msgs,
        )

    def handle_get_received_payments_for_squeak(self, request):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        limit = request.limit
        last_received_payment = message_to_received_payment(request.last_received_payment) if request.HasField(
            "last_received_payment") else None
        logger.info("""Handle get received payments with
        squeak_hash: {}
        limit: {}
        last_received_payment: {}
        """.format(
            squeak_hash,
            limit,
            last_received_payment,
        ))
        received_payments = self.squeak_controller.get_received_payments_for_squeak(
            squeak_hash,
            limit,
            last_received_payment,
        )
        logger.info(
            "Got number of received payments: {}".format(
                len(received_payments)
            )
        )
        received_payment_msgs = [
            received_payment_to_message(received_payment)
            for received_payment in received_payments
        ]
        return squeak_admin_pb2.GetReceivedPaymentsReply(
            received_payments=received_payment_msgs,
        )

    def handle_get_received_payments_for_pubkey(self, request):
        public_key_hex = request.pubkey
        limit = request.limit
        last_received_payment = message_to_received_payment(request.last_received_payment) if request.HasField(
            "last_received_payment") else None
        logger.info("""Handle get received payments with
        public_key: {}
        limit: {}
        last_received_payment: {}
        """.format(
            public_key_hex,
            limit,
            last_received_payment,
        ))
        public_key = SqueakPublicKey.from_bytes(bytes.fromhex(public_key_hex))
        received_payments = self.squeak_controller.get_received_payments_for_pubkey(
            public_key,
            limit,
            last_received_payment,
        )
        logger.info(
            "Got number of received payments: {}".format(
                len(received_payments)
            )
        )
        received_payment_msgs = [
            received_payment_to_message(received_payment)
            for received_payment in received_payments
        ]
        return squeak_admin_pb2.GetReceivedPaymentsReply(
            received_payments=received_payment_msgs,
        )

    def handle_get_received_payments_for_peer(self, request):
        peer_address = message_to_peer_address(request.peer_address)
        limit = request.limit
        last_received_payment = message_to_received_payment(request.last_received_payment) if request.HasField(
            "last_received_payment") else None
        logger.info("""Handle get received payments with
        peer_address: {}
        limit: {}
        last_received_payment: {}
        """.format(
            peer_address,
            limit,
            last_received_payment,
        ))
        received_payments = self.squeak_controller.get_received_payments_for_peer(
            peer_address,
            limit,
            last_received_payment,
        )
        logger.info(
            "Got number of received payments: {}".format(
                len(received_payments)
            )
        )
        received_payment_msgs = [
            received_payment_to_message(received_payment)
            for received_payment in received_payments
        ]
        return squeak_admin_pb2.GetReceivedPaymentsReply(
            received_payments=received_payment_msgs,
        )

    def handle_subscribe_received_payments(self, request, stopped):
        payment_index = request.payment_index
        logger.info(
            "Handle subscribe received payments with index: {}".format(
                payment_index)
        )
        received_payments_stream = self.squeak_controller.subscribe_received_payments(
            payment_index,
            stopped,
        )
        for received_payment in received_payments_stream:
            received_payment_msg = received_payment_to_message(
                received_payment)
            yield received_payment_msg

    def handle_get_network(self, request):
        logger.info("Handle get network")
        network = self.squeak_controller.get_network()
        return squeak_admin_pb2.GetNetworkReply(
            network=network,
        )

    def handle_get_payment_summary(self, request):
        logger.info("Handle get payment summary")
        payment_summary = self.squeak_controller.get_payment_summary()
        payment_summary_msg = payment_summary_to_message(
            payment_summary,
        )
        return squeak_admin_pb2.GetPaymentSummaryReply(
            payment_summary=payment_summary_msg,
        )

    def handle_get_payment_summary_for_squeak(self, request):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info("Handle get payment summary for squeak hash: {}".format(
            squeak_hash_str,
        ))
        payment_summary = self.squeak_controller.get_payment_summary_for_squeak(
            squeak_hash)
        payment_summary_msg = payment_summary_to_message(
            payment_summary,
        )
        return squeak_admin_pb2.GetPaymentSummaryForSqueakReply(
            payment_summary=payment_summary_msg,
        )

    def handle_get_payment_summary_for_pubkey(self, request):
        public_key_hex = request.pubkey
        logger.info("Handle get payment summary for pubkey: {}".format(
            public_key_hex,
        ))
        public_key = SqueakPublicKey.from_bytes(bytes.fromhex(public_key_hex))
        payment_summary = self.squeak_controller.get_payment_summary_for_pubkey(
            public_key)
        payment_summary_msg = payment_summary_to_message(
            payment_summary,
        )
        return squeak_admin_pb2.GetPaymentSummaryForPubkeyReply(
            payment_summary=payment_summary_msg,
        )

    def handle_get_payment_summary_for_peer(self, request):
        peer_address = message_to_peer_address(request.peer_address)
        logger.info("Handle get payment summary for peer address: {}".format(
            peer_address,
        ))
        payment_summary = self.squeak_controller.get_payment_summary_for_peer(
            peer_address)
        payment_summary_msg = payment_summary_to_message(
            payment_summary,
        )
        return squeak_admin_pb2.GetPaymentSummaryForPubkeyReply(
            payment_summary=payment_summary_msg,
        )

    def handle_reprocess_received_payments(self, request):
        logger.info("Handle reprocess received payments")
        self.squeak_controller.reprocess_received_payments()
        return squeak_admin_pb2.ReprocessReceivedPaymentsReply()

    def handle_like_squeak(self, request: squeak_admin_pb2.LikeSqueakRequest):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle like squeak with hash: {}".format(squeak_hash_str))
        self.squeak_controller.like_squeak(
            squeak_hash
        )
        return squeak_admin_pb2.LikeSqueakReply()

    def handle_unlike_squeak(self, request: squeak_admin_pb2.UnlikeSqueakRequest):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle unlike squeak with hash: {}".format(squeak_hash_str))
        self.squeak_controller.unlike_squeak(
            squeak_hash
        )
        return squeak_admin_pb2.UnlikeSqueakReply()

    def handle_get_liked_squeak_display_entries(self, request):
        limit = request.limit
        last_entry = message_to_squeak_entry(request.last_entry) if request.HasField(
            "last_entry") else None
        logger.info("""Handle get liked squeak display entries with
        limit: {}
        last_entry: {}
        """.format(
            limit,
            last_entry,
        ))
        squeak_entries = (
            self.squeak_controller.get_liked_squeak_entries(
                limit,
                last_entry,
            )
        )
        logger.info(
            "Got number of liked squeak entries: {}".format(
                len(squeak_entries)
            )
        )
        squeak_display_msgs = [
            squeak_entry_to_message(entry) for entry in squeak_entries
        ]
        return squeak_admin_pb2.GetLikedSqueakDisplaysReply(
            squeak_display_entries=squeak_display_msgs
        )

    def handle_subscribe_buy_offers(self, request, stopped):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle subscribe received offers for hash: {}".format(squeak_hash_str))
        received_offer_stream = self.squeak_controller.subscribe_received_offers_for_squeak(
            squeak_hash,
            stopped,
        )
        for offer in received_offer_stream:
            logger.info("Yielding received offer: {}".format(offer))
            offer_msg = received_offer_to_message(offer)
            yield squeak_admin_pb2.GetBuyOfferReply(
                offer=offer_msg,
            )

    def handle_subscribe_squeak_display(self, request, stopped):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle subscribe squeak display for hash: {}".format(squeak_hash_str))
        squeak_display_stream = self.squeak_controller.subscribe_squeak_entry(
            squeak_hash,
            stopped,
        )
        for squeak_display in squeak_display_stream:
            display_message = optional_squeak_entry_to_message(
                squeak_display)
            yield squeak_admin_pb2.GetSqueakDisplayReply(
                squeak_display_entry=display_message
            )

    def handle_subscribe_reply_squeak_displays(self, request, stopped):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle subscribe reply squeak displays for hash: {}".format(squeak_hash_str))
        squeak_display_stream = self.squeak_controller.subscribe_squeak_reply_entries(
            squeak_hash,
            stopped,
        )
        for squeak_display in squeak_display_stream:
            display_message = optional_squeak_entry_to_message(
                squeak_display)
            yield squeak_admin_pb2.GetSqueakDisplayReply(
                squeak_display_entry=display_message
            )

    def handle_subscribe_pubkey_squeak_displays(self, request, stopped):
        public_key_hex = request.pubkey
        logger.info(
            "Handle subscribe squeak displays for public key: {}".format(public_key_hex))
        public_key = SqueakPublicKey.from_bytes(bytes.fromhex(public_key_hex))
        squeak_display_stream = self.squeak_controller.subscribe_squeak_public_key_entries(
            public_key,
            stopped,
        )
        for squeak_display in squeak_display_stream:
            display_message = optional_squeak_entry_to_message(
                squeak_display)
            yield squeak_admin_pb2.GetSqueakDisplayReply(
                squeak_display_entry=display_message
            )

    def handle_subscribe_ancestor_squeak_displays(self, request, stopped):
        squeak_hash_str = request.squeak_hash
        squeak_hash = bytes.fromhex(squeak_hash_str)
        logger.info(
            "Handle subscribe ancestor squeak displays for hash: {}".format(squeak_hash_str))
        squeak_entries_stream = self.squeak_controller.subscribe_squeak_ancestor_entries(
            squeak_hash,
            stopped,
        )
        for squeak_entries in squeak_entries_stream:
            logger.info(
                "Got number of ancestor squeak entries: {}".format(
                    len(squeak_entries)
                )
            )
            squeak_display_msgs = [
                squeak_entry_to_message(entry) for entry in squeak_entries
            ]
            yield squeak_admin_pb2.GetAncestorSqueakDisplaysReply(
                squeak_display_entries=squeak_display_msgs
            )

    def handle_subscribe_squeak_displays(self, request, stopped):
        logger.info("Handle subscribe squeak displays")
        squeak_display_stream = self.squeak_controller.subscribe_squeak_entries(
            stopped,
        )
        for squeak_display in squeak_display_stream:
            display_message = optional_squeak_entry_to_message(
                squeak_display)
            yield squeak_admin_pb2.GetSqueakDisplayReply(
                squeak_display_entry=display_message
            )

    def handle_subscribe_timeline_squeak_displays(self, request, stopped):
        logger.info("Handle subscribe timeline squeak displays")
        squeak_display_stream = self.squeak_controller.subscribe_timeline_squeak_entries(
            stopped,
        )
        for squeak_display in squeak_display_stream:
            display_message = optional_squeak_entry_to_message(
                squeak_display)
            yield squeak_admin_pb2.GetSqueakDisplayReply(
                squeak_display_entry=display_message
            )

    def handle_get_external_address(self, request):
        logger.info("Handle get external address")
        external_address = self.squeak_controller.get_external_address()
        external_address_msg = peer_address_to_message(external_address)
        return squeak_admin_pb2.GetExternalAddressReply(
            peer_address=external_address_msg,
        )

    def handle_get_default_peer_port(self, request):
        logger.info("Handle get default peer port")
        default_peer_port = self.squeak_controller.get_default_peer_port()
        return squeak_admin_pb2.GetDefaultPeerPortReply(
            port=default_peer_port,
        )

    def handle_set_sell_price(self, request):
        sell_price_msat = request.price_msat
        logger.info("Handle set sell price msat to: {}".format(
            sell_price_msat,
        ))
        self.squeak_controller.set_sell_price_msat(sell_price_msat)
        return squeak_admin_pb2.SetSellPriceReply()

    def handle_clear_sell_price(self, request):
        logger.info("Handle clear sell price.")
        self.squeak_controller.clear_sell_price_msat()
        return squeak_admin_pb2.ClearSellPriceReply()

    def handle_get_sell_price(self, request):
        logger.info("Handle get sell price")
        sell_price_msat = self.squeak_controller.get_sell_price_msat()
        logger.info("sell price: {}".format(sell_price_msat))
        return squeak_admin_pb2.GetSellPriceReply(
            price_msat=sell_price_msat,
        )

    def handle_add_twitter_account(self, request):
        handle = request.handle
        profile_id = request.profile_id
        bearer_token = request.bearer_token
        logger.info("Handle add twitter account with handle: {}, profile_id: {}, bearer token: {}".format(
            handle,
            profile_id,
            bearer_token,
        ))
        twitter_account_id = self.squeak_controller.add_twitter_account(
            handle,
            profile_id,
            bearer_token,
        )
        return squeak_admin_pb2.AddTwitterAccountReply(
            twitter_account_id=twitter_account_id,
        )

    def handle_get_twitter_accounts(self, request):
        logger.info("Handle get twitter accounts")
        twitter_accounts = self.squeak_controller.get_twitter_accounts()
        logger.info("Got number of twitter accounts: {}".format(
            len(twitter_accounts)))
        twitter_account_msgs = [
            twitter_account_to_message(twitter_account)
            for twitter_account in twitter_accounts
        ]
        return squeak_admin_pb2.GetTwitterAccountsReply(
            twitter_accounts=twitter_account_msgs,
        )

    def handle_delete_twitter_account(self, request):
        twitter_account_id = request.twitter_account_id
        logger.info("Handle delete twitter account with id: {}".format(
            twitter_account_id,
        ))
        self.squeak_controller.delete_twitter_account(
            twitter_account_id,
        )
        return squeak_admin_pb2.DeleteTwitterAccountReply()
