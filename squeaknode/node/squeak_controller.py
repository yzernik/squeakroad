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
import threading
from typing import List
from typing import Optional

from squeak.core import CSqueak
from squeak.core.keys import SqueakPrivateKey
from squeak.core.keys import SqueakPublicKey

from squeaknode.core.download_result import DownloadResult
from squeaknode.core.lightning_address import LightningAddressHostPort
from squeaknode.core.offer import Offer
from squeaknode.core.payment_summary import PaymentSummary
from squeaknode.core.peer_address import Network
from squeaknode.core.peer_address import PeerAddress
from squeaknode.core.received_offer import ReceivedOffer
from squeaknode.core.received_payment import ReceivedPayment
from squeaknode.core.sent_payment import SentPayment
from squeaknode.core.squeak_entry import SqueakEntry
from squeaknode.core.squeak_peer import SqueakPeer
from squeaknode.core.squeak_profile import SqueakProfile
from squeaknode.core.squeaks import get_hash
from squeaknode.core.twitter_account_entry import TwitterAccountEntry
from squeaknode.node.received_payments_subscription_client import ReceivedPaymentsSubscriptionClient
from squeaknode.node.squeak_store import SqueakStore


logger = logging.getLogger(__name__)


class SqueakController:
    """Control plane for all actions on a node.

    """

    def __init__(
        self,
        squeak_store: SqueakStore,
        payment_processor,
        tweet_forwarder,
        network_controller,
        node_settings,
        config,
    ):
        self.squeak_store = squeak_store
        self.payment_processor = payment_processor
        self.tweet_forwarder = tweet_forwarder
        self.network_controller = network_controller
        self.node_settings = node_settings
        self.config = config

    def make_squeak(
            self,
            profile_id: int,
            content_str: str,
            replyto_hash: Optional[bytes],
            recipient_profile_id: Optional[int],
    ) -> Optional[bytes]:
        return self.squeak_store.make_squeak(
            profile_id,
            content_str,
            replyto_hash,
            recipient_profile_id,
        )

    def make_resqueak(
            self,
            profile_id: int,
            resqueaked_hash: bytes,
            replyto_hash: Optional[bytes],
    ) -> Optional[bytes]:
        return self.squeak_store.make_resqueak(
            profile_id,
            resqueaked_hash,
            replyto_hash,
        )

    def pay_offer(self, received_offer_id: int) -> int:
        return self.squeak_store.pay_offer(received_offer_id)

    def get_packaged_offer(
            self,
            squeak_hash: bytes,
            peer_address: PeerAddress,
    ) -> Optional[Offer]:
        lnd_external_address: Optional[LightningAddressHostPort] = None
        if self.config.lightning.external_host:
            lnd_external_address = LightningAddressHostPort(
                host=self.config.lightning.external_host,
                port=self.config.lightning.external_port,
            )
        price_msat = self.get_sell_price_msat()
        if price_msat == 0:
            return None
        return self.squeak_store.get_packaged_offer(
            squeak_hash,
            peer_address,
            price_msat,
            lnd_external_address,
        )

    def decrypt_private_squeak(
            self,
            squeak_hash: bytes,
            author_profile_id: Optional[int],
            recipient_profile_id: Optional[int],
    ):
        self.squeak_store.unlock_squeak(
            squeak_hash,
            author_profile_id=author_profile_id,
            recipient_profile_id=recipient_profile_id,
        )

    def get_squeak(self, squeak_hash: bytes) -> Optional[CSqueak]:
        return self.squeak_store.get_squeak(squeak_hash)

    def get_squeak_secret_key(self, squeak_hash: bytes) -> Optional[bytes]:
        return self.squeak_store.get_squeak_secret_key(squeak_hash)

    def delete_squeak(self, squeak_hash: bytes) -> None:
        self.squeak_store.delete_squeak(squeak_hash)

    def create_signing_profile(self, profile_name: str) -> int:
        return self.squeak_store.create_signing_profile(profile_name)

    def import_signing_profile(self, profile_name: str, private_key: SqueakPrivateKey) -> int:
        return self.squeak_store.import_signing_profile(profile_name, private_key)

    def create_contact_profile(self, profile_name: str, public_key: SqueakPublicKey) -> int:
        return self.squeak_store.create_contact_profile(profile_name, public_key)

    def get_profiles(self) -> List[SqueakProfile]:
        return self.squeak_store.get_profiles()

    def get_signing_profiles(self) -> List[SqueakProfile]:
        return self.squeak_store.get_signing_profiles()

    def get_contact_profiles(self) -> List[SqueakProfile]:
        return self.squeak_store.get_contact_profiles()

    def get_squeak_profile(self, profile_id: int) -> Optional[SqueakProfile]:
        return self.squeak_store.get_squeak_profile(profile_id)

    def get_squeak_profile_by_public_key(self, public_key: SqueakPublicKey) -> Optional[SqueakProfile]:
        return self.squeak_store.get_squeak_profile_by_public_key(public_key)

    def get_squeak_profile_by_name(self, name: str) -> Optional[SqueakProfile]:
        return self.squeak_store.get_squeak_profile_by_name(name)

    def set_squeak_profile_following(self, profile_id: int, following: bool) -> None:
        return self.squeak_store.set_squeak_profile_following(profile_id, following)

    def rename_squeak_profile(self, profile_id: int, profile_name: str) -> None:
        return self.squeak_store.rename_squeak_profile(profile_id, profile_name)

    def delete_squeak_profile(self, profile_id: int) -> None:
        return self.squeak_store.delete_squeak_profile(profile_id)

    def set_squeak_profile_image(self, profile_id: int, profile_image: bytes) -> None:
        return self.squeak_store.set_squeak_profile_image(profile_id, profile_image)

    def clear_squeak_profile_image(self, profile_id: int) -> None:
        return self.squeak_store.clear_squeak_profile_image(profile_id)

    def get_squeak_profile_private_key(self, profile_id: int) -> bytes:
        return self.squeak_store.get_squeak_profile_private_key(profile_id)

    def create_peer(self, peer_name: str, peer_address: PeerAddress):
        return self.squeak_store.create_peer(
            peer_name,
            peer_address,
            self.config.server.port or 0,
        )

    def get_peer(self, peer_id: int) -> Optional[SqueakPeer]:
        return self.squeak_store.get_peer(peer_id)

    def get_peer_by_address(self, peer_address: PeerAddress) -> Optional[SqueakPeer]:
        return self.squeak_store.get_peer_by_address(peer_address)

    def get_peers(self):
        return self.squeak_store.get_peers()

    def get_autoconnect_peers(self) -> List[SqueakPeer]:
        return self.squeak_store.get_autoconnect_peers()

    def set_peer_autoconnect(self, peer_id: int, autoconnect: bool):
        return self.squeak_store.set_peer_autoconnect(peer_id, autoconnect)

    def set_peer_share_for_free(self, peer_id: int, share_for_free: bool):
        return self.squeak_store.set_peer_share_for_free(peer_id, share_for_free)

    def rename_peer(self, peer_id: int, peer_name: str):
        return self.squeak_store.rename_peer(peer_id, peer_name)

    def delete_peer(self, peer_id: int):
        return self.squeak_store.delete_peer(peer_id)

    def get_received_offers(self, squeak_hash: bytes) -> List[ReceivedOffer]:
        return self.squeak_store.get_received_offers(squeak_hash)

    def get_received_offer(self, received_offer_id: int) -> Optional[ReceivedOffer]:
        return self.squeak_store.get_received_offer(received_offer_id)

    def get_sent_payments(
            self,
            limit: int,
            last_sent_payment: Optional[SentPayment],
    ) -> List[SentPayment]:
        return self.squeak_store.get_sent_payments(
            limit,
            last_sent_payment,
        )

    def get_sent_payments_for_squeak(
            self,
            squeak_hash: bytes,
            limit: int,
            last_sent_payment: Optional[SentPayment],
    ) -> List[SentPayment]:
        return self.squeak_store.get_sent_payments_for_squeak(
            squeak_hash,
            limit,
            last_sent_payment,
        )

    def get_sent_payments_for_pubkey(
            self,
            pubkey: SqueakPublicKey,
            limit: int,
            last_sent_payment: Optional[SentPayment],
    ) -> List[SentPayment]:
        return self.squeak_store.get_sent_payments_for_pubkey(
            pubkey,
            limit,
            last_sent_payment,
        )

    def get_sent_payments_for_peer(
            self,
            peer_address: PeerAddress,
            limit: int,
            last_sent_payment: Optional[SentPayment],
    ) -> List[SentPayment]:
        return self.squeak_store.get_sent_payments_for_peer(
            peer_address,
            limit,
            last_sent_payment,
        )

    def get_sent_payment(self, sent_payment_id: int) -> Optional[SentPayment]:
        return self.squeak_store.get_sent_payment(sent_payment_id)

    def get_sent_offers(self):
        return self.squeak_store.get_sent_offers()

    def get_received_payments(
            self,
            limit: int,
            last_received_payment: Optional[ReceivedPayment],
    ) -> List[ReceivedPayment]:
        return self.squeak_store.get_received_payments(
            limit,
            last_received_payment,
        )

    def get_received_payments_for_squeak(
            self,
            squeak_hash: bytes,
            limit: int,
            last_received_payment: Optional[ReceivedPayment],
    ) -> List[ReceivedPayment]:
        return self.squeak_store.get_received_payments_for_squeak(
            squeak_hash,
            limit,
            last_received_payment,
        )

    def get_received_payments_for_pubkey(
            self,
            pubkey: SqueakPublicKey,
            limit: int,
            last_received_payment: Optional[ReceivedPayment],
    ) -> List[ReceivedPayment]:
        return self.squeak_store.get_received_payments_for_pubkey(
            pubkey,
            limit,
            last_received_payment,
        )

    def get_received_payments_for_peer(
            self,
            peer_address: PeerAddress,
            limit: int,
            last_received_payment: Optional[ReceivedPayment],
    ) -> List[ReceivedPayment]:
        return self.squeak_store.get_received_payments_for_peer(
            peer_address,
            limit,
            last_received_payment,
        )

    def delete_all_expired_offers(self):
        self.squeak_store.delete_all_expired_offers()

    def subscribe_received_payments(self, initial_index: int, stopped: threading.Event):
        with ReceivedPaymentsSubscriptionClient(
            self.squeak_store,
            initial_index,
            stopped,
        ).open_subscription() as client:
            yield from client.get_received_payments()

    def get_network(self) -> str:
        return self.config.node.network

    def get_squeak_entry(self, squeak_hash: bytes) -> Optional[SqueakEntry]:
        return self.squeak_store.get_squeak_entry(squeak_hash)

    def download_single_squeak(self, squeak_hash: bytes) -> DownloadResult:
        self.network_controller.download_single_squeak(squeak_hash)
        return DownloadResult(1, 1, 0, 9999)

    def download_single_squeak_secret_key(self, squeak_hash: bytes) -> DownloadResult:
        self.network_controller.download_single_squeak_secret_key(squeak_hash)
        return DownloadResult(1, 1, 0, 9999)

    def get_timeline_squeak_entries(
            self,
            limit: int,
            last_entry: Optional[SqueakEntry],
    ) -> List[SqueakEntry]:
        return self.squeak_store.get_timeline_squeak_entries(limit, last_entry)

    def get_liked_squeak_entries(
            self,
            limit: int,
            last_entry: Optional[SqueakEntry],
    ) -> List[SqueakEntry]:
        return self.squeak_store.get_liked_squeak_entries(limit, last_entry)

    def lookup_squeaks(
            self,
            public_keys: List[SqueakPublicKey],
            min_block: Optional[int],
            max_block: Optional[int],
            reply_to_hash: Optional[bytes],
    ) -> List[bytes]:
        return self.squeak_store.lookup_squeaks(
            public_keys,
            min_block,
            max_block,
            reply_to_hash,
        )

    def get_squeak_entries_for_public_key(
            self,
            public_key: SqueakPublicKey,
            limit: int,
            last_entry: Optional[SqueakEntry],
    ) -> List[SqueakEntry]:
        return self.squeak_store.get_squeak_entries_for_public_key(
            public_key,
            limit,
            last_entry,
        )

    def get_squeak_entries_for_text_search(
            self,
            search_text: str,
            limit: int,
            last_entry: Optional[SqueakEntry],
    ) -> List[SqueakEntry]:
        return self.squeak_store.get_squeak_entries_for_text_search(
            search_text,
            limit,
            last_entry,
        )

    def get_ancestor_squeak_entries(self, squeak_hash: bytes) -> List[SqueakEntry]:
        return self.squeak_store.get_ancestor_squeak_entries(squeak_hash)

    def get_reply_squeak_entries(
            self,
            squeak_hash: bytes,
            limit: int,
            last_entry: Optional[SqueakEntry],
    ) -> List[SqueakEntry]:
        return self.squeak_store.get_reply_squeak_entries(
            squeak_hash,
            limit,
            last_entry,
        )

    def get_payment_summary(self) -> PaymentSummary:
        received_payment_summary = self.squeak_store.get_received_payment_summary()
        sent_payment_summary = self.squeak_store.get_sent_payment_summary()
        return PaymentSummary(
            sent_payment_summary=sent_payment_summary,
            received_payment_summary=received_payment_summary,
        )

    def get_payment_summary_for_squeak(self, squeak_hash: bytes) -> PaymentSummary:
        received_payment_summary = self.squeak_store.get_received_payment_summary_for_squeak(
            squeak_hash)
        sent_payment_summary = self.squeak_store.get_sent_payment_summary_for_squeak(
            squeak_hash)
        return PaymentSummary(
            sent_payment_summary=sent_payment_summary,
            received_payment_summary=received_payment_summary,
        )

    def get_payment_summary_for_pubkey(self, pubkey: SqueakPublicKey) -> PaymentSummary:
        received_payment_summary = self.squeak_store.get_received_payment_summary_for_pubkey(
            pubkey)
        sent_payment_summary = self.squeak_store.get_sent_payment_summary_for_pubkey(
            pubkey)
        return PaymentSummary(
            sent_payment_summary=sent_payment_summary,
            received_payment_summary=received_payment_summary,
        )

    def get_payment_summary_for_peer(self, peer_address: PeerAddress) -> PaymentSummary:
        received_payment_summary = self.squeak_store.get_received_payment_summary_for_peer(
            peer_address)
        sent_payment_summary = self.squeak_store.get_sent_payment_summary_for_peer(
            peer_address)
        return PaymentSummary(
            sent_payment_summary=sent_payment_summary,
            received_payment_summary=received_payment_summary,
        )

    # def get_received_payment_summary(self) -> ReceivedPaymentSummary:
    #     return self.squeak_store.get_received_payment_summary()

    # def get_received_payment_summary_for_squeak(self, squeak_hash: bytes) -> ReceivedPaymentSummary:
    #     return self.squeak_store.get_received_payment_summary_for_squeak(squeak_hash)

    # def get_received_payment_summary_for_pubkey(self, pubkey: SqueakPublicKey) -> ReceivedPaymentSummary:
    #     return self.squeak_store.get_received_payment_summary_for_pubkey(pubkey)

    # def get_sent_payment_summary(self) -> SentPaymentSummary:
    #     return self.squeak_store.get_sent_payment_summary()

    # def get_sent_payment_summary_for_squeak(self, squeak_hash: bytes) -> SentPaymentSummary:
    #     return self.squeak_store.get_sent_payment_summary_for_squeak(squeak_hash)

    # def get_sent_payment_summary_for_pubkey(self, pubkey: SqueakPublicKey) -> SentPaymentSummary:
    #     return self.squeak_store.get_sent_payment_summary_for_pubkey(pubkey)

    def reprocess_received_payments(self) -> None:
        self.squeak_store.clear_received_payment_settle_indices()
        self.payment_processor.start_processing()

    def delete_old_squeaks(self):
        return self.squeak_store.delete_old_squeaks()

    def like_squeak(self, squeak_hash: bytes):
        self.squeak_store.like_squeak(squeak_hash)

    def unlike_squeak(self, squeak_hash: bytes):
        return self.squeak_store.unlike_squeak(squeak_hash)

    def subscribe_new_squeaks(self, stopped: threading.Event):
        yield from self.squeak_store.subscribe_new_squeaks(stopped)

    def subscribe_new_secret_keys(self, stopped: threading.Event):
        yield from self.squeak_store.subscribe_new_secret_keys(stopped)

    def subscribe_follows(self, stopped: threading.Event):
        yield from self.squeak_store.subscribe_follows(stopped)

    def subscribe_received_offers_for_squeak(self, squeak_hash: bytes, stopped: threading.Event):
        yield from self.squeak_store.subscribe_received_offers_for_squeak(
            squeak_hash,
            stopped,
        )

    def subscribe_squeak_entry(self, squeak_hash: bytes, stopped: threading.Event):
        for item in self.squeak_store.subscribe_new_squeaks(stopped):
            if squeak_hash == get_hash(item):
                yield self.get_squeak_entry(squeak_hash)

    def subscribe_squeak_reply_entries(self, squeak_hash: bytes, stopped: threading.Event):
        for item in self.squeak_store.subscribe_new_squeaks(stopped):
            if squeak_hash == item.hashReplySqk:
                reply_hash = get_hash(item)
                yield self.get_squeak_entry(reply_hash)

    def subscribe_squeak_public_key_entries(self, public_key: SqueakPublicKey, stopped: threading.Event):
        for item in self.squeak_store.subscribe_new_squeaks(stopped):
            if public_key == item.GetPubKey():
                squeak_hash = get_hash(item)
                yield self.get_squeak_entry(squeak_hash)

    def subscribe_squeak_ancestor_entries(self, squeak_hash: bytes, stopped: threading.Event):
        for item in self.squeak_store.subscribe_new_squeaks(stopped):
            if squeak_hash == get_hash(item):
                yield self.get_ancestor_squeak_entries(squeak_hash)

    def subscribe_squeak_entries(self, stopped: threading.Event):
        for item in self.squeak_store.subscribe_new_squeaks(stopped):
            squeak_hash = get_hash(item)
            yield self.get_squeak_entry(squeak_hash)

    def subscribe_timeline_squeak_entries(self, stopped: threading.Event):
        for item in self.squeak_store.subscribe_new_squeaks(stopped):
            followed_public_keys = self.squeak_store.get_followed_public_keys()
            if item.GetPubKey() in set(followed_public_keys):
                squeak_hash = get_hash(item)
                yield self.get_squeak_entry(squeak_hash)

    def get_external_address(self) -> PeerAddress:
        return PeerAddress(
            network=Network.IPV4,
            host=self.config.server.external_address or '',
            port=self.config.server.external_port or 0,
        )

    def get_default_peer_port(self) -> int:
        return 0  # TODO: remove default port method.

    def set_sell_price_msat(self, sell_price_msat: int) -> None:
        self.node_settings.set_sell_price_msat(sell_price_msat)

    def clear_sell_price_msat(self) -> None:
        self.node_settings.clear_sell_price_msat()

    def get_sell_price_msat(self) -> int:
        configured_price = self.node_settings.get_sell_price_msat()
        if configured_price is None:
            return self.config.node.price_msat
        return configured_price

    def get_default_sell_price_msat(self) -> int:
        return self.config.node.price_msat

    def add_twitter_account(self, handle: str, profile_id: int, bearer_token: str) -> Optional[int]:
        twitter_account_id = self.squeak_store.add_twitter_account(
            handle,
            profile_id,
            bearer_token,
        )
        self.tweet_forwarder.restart()
        return twitter_account_id

    def get_twitter_accounts(self) -> List[TwitterAccountEntry]:
        accounts = self.squeak_store.get_twitter_accounts()
        return [
            account._replace(
                is_forwarding=self.tweet_forwarder.is_processing(
                    account.handle,
                ),
            )
            for account in accounts
        ]

    def delete_twitter_account(self, twitter_account_id: int) -> None:
        self.squeak_store.delete_twitter_account(twitter_account_id)
        self.tweet_forwarder.restart()
