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
from typing import Iterator
from typing import List
from typing import Optional

from squeak.core import CBaseSqueak
from squeak.core import CheckSqueak
from squeak.core import CheckSqueakSecretKey
from squeak.core import CSqueak
from squeak.core.keys import SqueakPrivateKey
from squeak.core.keys import SqueakPublicKey

from squeaknode.core.lightning_address import LightningAddressHostPort
from squeaknode.core.offer import Offer
from squeaknode.core.peer_address import PeerAddress
from squeaknode.core.peers import create_saved_peer
from squeaknode.core.profiles import create_contact_profile
from squeaknode.core.profiles import create_signing_profile
from squeaknode.core.profiles import get_profile_private_key
from squeaknode.core.received_offer import ReceivedOffer
from squeaknode.core.received_payment import ReceivedPayment
from squeaknode.core.received_payment_summary import ReceivedPaymentSummary
from squeaknode.core.sent_offer import SentOffer
from squeaknode.core.sent_payment import SentPayment
from squeaknode.core.sent_payment_summary import SentPaymentSummary
from squeaknode.core.squeak_core import SqueakCore
from squeaknode.core.squeak_entry import SqueakEntry
from squeaknode.core.squeak_peer import SqueakPeer
from squeaknode.core.squeak_profile import SqueakProfile
from squeaknode.core.squeak_user import SqueakUser
from squeaknode.db.squeak_db import SqueakDb


logger = logging.getLogger(__name__)


class SqueakStore:

    def __init__(
        self,
        squeak_db: SqueakDb,
        squeak_core: SqueakCore,
        max_squeaks,
        max_squeaks_per_public_key_per_block,
        squeak_retention_s,
        received_offer_retention_s,
        sent_offer_retention_s,
    ):
        self.squeak_db = squeak_db
        self.squeak_core = squeak_core
        self.max_squeaks = max_squeaks
        self.max_squeaks_per_public_key_per_block = max_squeaks_per_public_key_per_block
        self.squeak_retention_s = squeak_retention_s
        self.received_offer_retention_s = received_offer_retention_s
        self.sent_offer_retention_s = sent_offer_retention_s

    def make_squeak(
            self,
            profile_id: int,
            content_str: str,
    ) -> Optional[bytes]:
        squeak_profile = self.get_squeak_profile(profile_id)
        if squeak_profile is None:
            raise Exception("Profile with id {} not found.".format(
                profile_id,
            ))
        squeak, secret_key = self.squeak_core.make_squeak(
            squeak_profile,
            content_str,
            None,
            None,
        )
        inserted_squeak_hash = self.save_squeak(squeak)
        if inserted_squeak_hash is None:
            raise Exception("Failed to save squeak.")
        self.save_secret_key(inserted_squeak_hash, secret_key)
        if squeak.is_private_message:
            self.unlock_squeak(
                inserted_squeak_hash,
                author_profile_id=profile_id,
            )
        return inserted_squeak_hash

    def save_squeak(self, base_squeak: CBaseSqueak) -> Optional[bytes]:
        # Check if the squeak is valid context free.
        CheckSqueak(base_squeak)
        # Get the block header.
        block_header = self.squeak_core.get_block_header(base_squeak)
        # Check if limit exceeded.
        if self.squeak_db.get_number_of_squeaks() >= self.max_squeaks:
            raise Exception("Exceeded max number of squeaks.")
        # TODO: Check if limit per public key per block is exceeded.
        if self.squeak_db.number_of_squeaks_with_public_key_with_block_height(
                base_squeak.GetPubKey(),
                base_squeak.nBlockHeight,
        ) >= self.max_squeaks_per_public_key_per_block:
            raise Exception(
                "Exceeded max number of squeaks per public key per block.")
        # Insert the squeak in db.
        if isinstance(base_squeak, CSqueak):
            inserted_squeak_hash = self.squeak_db.insert_squeak(
                base_squeak,
                block_header,
            )
        if inserted_squeak_hash is None:
            return None
        logger.info("Saved squeak: {}".format(
            inserted_squeak_hash.hex(),
        ))
        return inserted_squeak_hash

    def save_secret_key(self, squeak_hash: bytes, secret_key: bytes):
        squeak = self.squeak_db.get_squeak(squeak_hash)
        if squeak is None:
            raise Exception("Squeakdoes not exist.")
        CheckSqueakSecretKey(squeak, secret_key)
        self.squeak_db.set_squeak_secret_key(
            squeak_hash,
            secret_key,
        )
        logger.info("Saved squeak secret key: {}".format(
            squeak_hash.hex(),
        ))
        # Unlock the squeak if it is not private.
        if not squeak.is_private_message:
            self.unlock_squeak(squeak_hash)

    def unlock_squeak(
            self,
            squeak_hash: bytes,
            author_profile_id: Optional[int] = None,
            recipient_profile_id: Optional[int] = None,
    ):
        squeak = self.squeak_db.get_squeak(squeak_hash)
        secret_key = self.squeak_db.get_squeak_secret_key(squeak_hash)
        if squeak is None:
            raise Exception("Squeakdoes not exist.")
        if secret_key is None:
            raise Exception("Secret key does not exist.")
        if recipient_profile_id:
            recipient_profile = self.squeak_db.get_profile(
                recipient_profile_id)
            if recipient_profile is None:
                raise Exception("Recipient profile does not exist.")
            decrypted_content = self.squeak_core.get_decrypted_content(
                squeak,
                secret_key,
                recipient_profile=recipient_profile,
            )
        elif author_profile_id:
            author_profile = self.squeak_db.get_profile(
                author_profile_id)
            if author_profile is None:
                raise Exception("Author profile does not exist.")
            decrypted_content = self.squeak_core.get_decrypted_content(
                squeak,
                secret_key,
                author_profile=author_profile,
            )
        else:
            decrypted_content = self.squeak_core.get_decrypted_content(
                squeak,
                secret_key,
            )
        self.squeak_db.set_squeak_decrypted_content(
            squeak_hash,
            decrypted_content,
        )
        logger.info("Unlocked squeak content: {}".format(
            squeak_hash.hex(),
        ))

    def pay_offer(self, received_offer_id: int) -> int:
        received_offer = self.get_received_offer(
            received_offer_id,
        )
        if received_offer is None:
            raise Exception("Received offer with id {} not found.".format(
                received_offer_id,
            ))
        logger.info("Paying received offer: {}".format(received_offer))
        sent_payment = self.squeak_core.pay_offer(received_offer)
        sent_payment_id = self.save_sent_payment(sent_payment)
        self.mark_received_offer_paid(
            sent_payment.payment_hash,
        )
        self.save_secret_key(
            received_offer.squeak_hash,
            sent_payment.secret_key,
        )
        return sent_payment_id

    def get_squeak(self, squeak_hash: bytes) -> Optional[CSqueak]:
        # TODO: remove this after squeak protocol struct stabilizes.
        try:
            return self.squeak_db.get_squeak(squeak_hash)
        except Exception:
            return None

    def get_squeak_secret_key(self, squeak_hash: bytes) -> Optional[bytes]:
        return self.squeak_db.get_squeak_secret_key(squeak_hash)

    def delete_squeak(self, squeak_hash: bytes) -> None:
        self.squeak_db.delete_squeak(squeak_hash)

    def save_sent_offer(self, sent_offer: SentOffer) -> int:
        return self.squeak_db.insert_sent_offer(sent_offer)

    def get_sent_offer_for_peer(
            self,
            squeak_hash: bytes,
            peer_address: PeerAddress,
            price_msat: int,
    ) -> Optional[SentOffer]:
        squeak = self.get_squeak(squeak_hash)
        secret_key = self.get_squeak_secret_key(squeak_hash)
        if squeak is None or secret_key is None:
            return None
        try:
            sent_offer = self.squeak_core.create_offer(
                squeak,
                secret_key,
                peer_address,
                price_msat,
            )
        except Exception:
            logger.exception("Failed to create offer.")
            return None
        self.save_sent_offer(sent_offer)
        return sent_offer

    # TODO: remove this method. Do this logic in squeakcontroller.
    def get_packaged_offer(
            self,
            squeak_hash: bytes,
            peer_address: PeerAddress,
            price_msat: int,
            lnd_external_address: Optional[LightningAddressHostPort],
    ) -> Optional[Offer]:
        sent_offer = self.get_sent_offer_for_peer(
            squeak_hash,
            peer_address,
            price_msat,
        )
        if sent_offer is None:
            return None
        return self.squeak_core.package_offer(
            sent_offer,
            lnd_external_address,
        )

    def create_signing_profile(self, profile_name: str) -> int:
        squeak_profile = create_signing_profile(
            profile_name,
        )
        profile_id = self.squeak_db.insert_profile(squeak_profile)
        return profile_id

    def import_signing_profile(self, profile_name: str, private_key: SqueakPrivateKey) -> int:
        squeak_profile = create_signing_profile(
            profile_name,
            private_key,
        )
        profile_id = self.squeak_db.insert_profile(squeak_profile)
        return profile_id

    def create_contact_profile(self, profile_name: str, public_key: SqueakPublicKey) -> int:
        squeak_profile = create_contact_profile(
            profile_name,
            public_key,
        )
        profile_id = self.squeak_db.insert_profile(squeak_profile)
        return profile_id

    def get_profiles(self) -> List[SqueakProfile]:
        return self.squeak_db.get_profiles()

    def get_signing_profiles(self) -> List[SqueakProfile]:
        return self.squeak_db.get_signing_profiles()

    def get_contact_profiles(self) -> List[SqueakProfile]:
        return self.squeak_db.get_contact_profiles()

    def get_squeak_profile(self, profile_id: int) -> Optional[SqueakProfile]:
        return self.squeak_db.get_profile(profile_id)

    def get_squeak_profile_by_public_key(self, public_key: SqueakPublicKey) -> Optional[SqueakProfile]:
        return self.squeak_db.get_profile_by_public_key(public_key)

    def get_squeak_profile_by_name(self, name: str) -> Optional[SqueakProfile]:
        return self.squeak_db.get_profile_by_name(name)

    def set_squeak_profile_following(self, profile_id: int, following: bool) -> None:
        self.squeak_db.set_profile_following(profile_id, following)

    def rename_squeak_profile(self, profile_id: int, profile_name: str) -> None:
        self.squeak_db.set_profile_name(profile_id, profile_name)

    def delete_squeak_profile(self, profile_id: int) -> None:
        self.squeak_db.delete_profile(profile_id)

    def set_squeak_profile_image(self, profile_id: int, profile_image: bytes) -> None:
        self.squeak_db.set_profile_image(profile_id, profile_image)

    def clear_squeak_profile_image(self, profile_id: int) -> None:
        self.squeak_db.set_profile_image(profile_id, None)

    def yield_received_payments_from_index(self, start_index: int = 0) -> Iterator[ReceivedPayment]:
        yield from self.squeak_db.yield_received_payments_from_index(start_index=start_index)

    def get_squeak_profile_private_key(self, profile_id: int) -> bytes:
        profile = self.get_squeak_profile(profile_id)
        if profile is None:
            raise Exception("Profile with id: {} does not exist.".format(
                profile_id,
            ))
        return get_profile_private_key(profile)

    def create_peer(self, peer_name: str, peer_address: PeerAddress, default_peer_port):
        squeak_peer = create_saved_peer(
            peer_name,
            peer_address,
            default_peer_port,
        )
        return self.squeak_db.insert_peer(squeak_peer)

    def get_peer(self, peer_id: int) -> Optional[SqueakPeer]:
        return self.squeak_db.get_peer(peer_id)

    def get_peer_by_address(self, peer_address: PeerAddress) -> Optional[SqueakPeer]:
        return self.squeak_db.get_peer_by_address(peer_address)

    def get_peers(self):
        return self.squeak_db.get_peers()

    def get_autoconnect_peers(self) -> List[SqueakPeer]:
        return self.squeak_db.get_autoconnect_peers()

    def set_peer_autoconnect(self, peer_id: int, autoconnect: bool):
        self.squeak_db.set_peer_autoconnect(peer_id, autoconnect)

    def set_peer_share_for_free(self, peer_id: int, share_for_free: bool):
        self.squeak_db.set_peer_share_for_free(peer_id, share_for_free)

    def rename_peer(self, peer_id: int, peer_name: str):
        self.squeak_db.set_peer_name(peer_id, peer_name)

    def delete_peer(self, peer_id: int):
        self.squeak_db.delete_peer(peer_id)

    def get_received_offers(self, squeak_hash: bytes) -> List[ReceivedOffer]:
        return self.squeak_db.get_received_offers(squeak_hash)

    def get_received_offer(self, received_offer_id: int) -> Optional[ReceivedOffer]:
        return self.squeak_db.get_received_offer(received_offer_id)

    def save_sent_payment(self, sent_payment: SentPayment) -> int:
        return self.squeak_db.insert_sent_payment(sent_payment)

    def get_sent_payments(
            self,
            limit: int,
            last_sent_payment: Optional[SentPayment],
    ) -> List[SentPayment]:
        return self.squeak_db.get_sent_payments(
            limit,
            last_sent_payment,
        )

    def get_sent_payments_for_squeak(
            self,
            squeak_hash: bytes,
            limit: int,
            last_sent_payment: Optional[SentPayment],
    ) -> List[SentPayment]:
        return self.squeak_db.get_sent_payments_for_squeak(
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
        return self.squeak_db.get_sent_payments_for_pubkey(
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
        return self.squeak_db.get_sent_payments_for_peer(
            peer_address,
            limit,
            last_sent_payment,
        )

    def get_sent_payment(self, sent_payment_id: int) -> Optional[SentPayment]:
        return self.squeak_db.get_sent_payment(sent_payment_id)

    def get_sent_offers(self):
        return self.squeak_db.get_sent_offers()

    def mark_received_offer_paid(self, payment_hash: bytes) -> None:
        self.squeak_db.set_received_offer_paid(payment_hash, True)

    def get_received_payments(
            self,
            limit: int,
            last_received_payment: Optional[ReceivedPayment],
    ) -> List[ReceivedPayment]:
        return self.squeak_db.get_received_payments(
            limit,
            last_received_payment,
        )

    def get_received_payments_for_squeak(
            self,
            squeak_hash: bytes,
            limit: int,
            last_received_payment: Optional[ReceivedPayment],
    ) -> List[ReceivedPayment]:
        return self.squeak_db.get_received_payments_for_squeak(
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
        return self.squeak_db.get_received_payments_for_pubkey(
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
        return self.squeak_db.get_received_payments_for_peer(
            peer_address,
            limit,
            last_received_payment,
        )

    def delete_all_expired_offers(self):
        self.delete_all_expired_received_offers()
        self.delete_all_expired_sent_offers()

    def delete_all_expired_received_offers(self):
        num_expired_received_offers = self.squeak_db.delete_expired_received_offers(
            self.received_offer_retention_s,
        )
        if num_expired_received_offers > 0:
            logger.info("Deleted number of expired received offers: {}".format(
                num_expired_received_offers))

    def delete_all_expired_sent_offers(self):
        num_expired_sent_offers = self.squeak_db.delete_expired_sent_offers(
            self.sent_offer_retention_s,
        )
        if num_expired_sent_offers > 0:
            logger.info(
                "Deleted number of expired sent offers: {}".format(
                    num_expired_sent_offers)
            )

    def get_squeak_entry(self, squeak_hash: bytes) -> Optional[SqueakEntry]:
        return self.squeak_db.get_squeak_entry(squeak_hash)

    def get_timeline_squeak_entries(
            self,
            limit: int,
            last_entry: Optional[SqueakEntry],
    ) -> List[SqueakEntry]:
        return self.squeak_db.get_timeline_squeak_entries(
            limit,
            last_entry,
        )

    def get_squeak_entries_for_public_key(
            self,
            public_key: SqueakPublicKey,
            limit: int,
            last_entry: Optional[SqueakEntry],
    ) -> List[SqueakEntry]:
        return self.squeak_db.get_squeak_entries_for_public_key(
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
        return self.squeak_db.get_squeak_entries_for_text_search(
            search_text,
            limit,
            last_entry,
        )

    def save_received_offer(self, received_offer: ReceivedOffer) -> Optional[int]:
        received_offer_id = self.squeak_db.insert_received_offer(
            received_offer)
        if received_offer_id is None:
            return None
        logger.info("Saved received offer: {}".format(received_offer))
        received_offer = received_offer._replace(
            received_offer_id=received_offer_id)
        return received_offer_id

    def handle_offer(self, squeak: CSqueak, offer: Offer, peer_address: PeerAddress):
        received_offer = self.squeak_core.unpack_offer(
            squeak,
            offer,
            peer_address,
        )
        self.save_received_offer(received_offer)

    def get_followed_public_keys(self) -> List[SqueakPublicKey]:
        followed_profiles = self.squeak_db.get_following_profiles()
        return [profile.public_key for profile in followed_profiles]

    def get_received_payment_summary(self) -> ReceivedPaymentSummary:
        return self.squeak_db.get_received_payment_summary()

    def get_received_payment_summary_for_squeak(self, squeak_hash: bytes) -> ReceivedPaymentSummary:
        return self.squeak_db.get_received_payment_summary_for_squeak(squeak_hash)

    def get_received_payment_summary_for_pubkey(self, pubkey: SqueakPublicKey) -> ReceivedPaymentSummary:
        return self.squeak_db.get_received_payment_summary_for_pubkey(pubkey)

    def get_received_payment_summary_for_peer(self, peer_address: PeerAddress) -> ReceivedPaymentSummary:
        return self.squeak_db.get_received_payment_summary_for_peer(peer_address)

    def get_sent_payment_summary(self) -> SentPaymentSummary:
        return self.squeak_db.get_sent_payment_summary()

    def get_sent_payment_summary_for_squeak(self, squeak_hash: bytes) -> SentPaymentSummary:
        return self.squeak_db.get_sent_payment_summary_for_squeak(squeak_hash)

    def get_sent_payment_summary_for_pubkey(self, pubkey: SqueakPublicKey) -> SentPaymentSummary:
        return self.squeak_db.get_sent_payment_summary_for_pubkey(pubkey)

    def get_sent_payment_summary_for_peer(self, peer_address: PeerAddress) -> SentPaymentSummary:
        return self.squeak_db.get_sent_payment_summary_for_peer(peer_address)

    def clear_received_payment_settle_indices(self) -> None:
        self.squeak_db.clear_received_payment_settle_indices()

    def delete_old_squeaks(self):
        squeaks_to_delete = self.squeak_db.get_old_squeaks_to_delete(
            self.squeak_retention_s,
        )
        for squeak_hash in squeaks_to_delete:
            self.squeak_db.delete_squeak(
                squeak_hash,
            )
            logger.info("Deleted squeak: {}".format(
                squeak_hash.hex(),
            ))

    def like_squeak(self, squeak_hash: bytes):
        logger.info("Liking squeak: {}".format(
            squeak_hash.hex(),
        ))
        self.squeak_db.set_squeak_liked(
            squeak_hash,
        )

    def unlike_squeak(self, squeak_hash: bytes):
        logger.info("Unliking squeak: {}".format(
            squeak_hash.hex(),
        ))
        self.squeak_db.set_squeak_unliked(
            squeak_hash,
        )

    def get_user_by_username(self, username: str) -> Optional[SqueakUser]:
        return self.squeak_db.get_user_by_username(username)

    def create_user(self, username: str, password_hash: str) -> int:
        squeak_user = SqueakUser(
            username=username,
            password_hash=password_hash,
        )
        return self.squeak_db.insert_user(squeak_user)
