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
import time
from contextlib import contextmanager
from typing import Iterator
from typing import List
from typing import Optional

import sqlalchemy
from bitcoin.core import CBlockHeader
from sqlalchemy import func
from sqlalchemy import not_
from sqlalchemy import or_
from sqlalchemy.sql import select
from sqlalchemy.sql import tuple_
from squeak.core import CBaseSqueak
from squeak.core import CResqueak
from squeak.core import CSqueak
from squeak.core.keys import SqueakPrivateKey
from squeak.core.keys import SqueakPublicKey

from squeaknode.core.lightning_address import LightningAddressHostPort
from squeaknode.core.peer_address import Network
from squeaknode.core.peer_address import PeerAddress
from squeaknode.core.received_offer import ReceivedOffer
from squeaknode.core.received_payment import ReceivedPayment
from squeaknode.core.received_payment_summary import ReceivedPaymentSummary
from squeaknode.core.sent_offer import SentOffer
from squeaknode.core.sent_payment import SentPayment
from squeaknode.core.sent_payment_summary import SentPaymentSummary
from squeaknode.core.squeak_entry import SqueakEntry
from squeaknode.core.squeak_peer import SqueakPeer
from squeaknode.core.squeak_profile import SqueakProfile
from squeaknode.core.squeak_user import SqueakUser
from squeaknode.core.squeaks import get_hash
from squeaknode.core.twitter_account import TwitterAccount
from squeaknode.core.twitter_account_entry import TwitterAccountEntry
from squeaknode.core.user_config import UserConfig
from squeaknode.db.exception import SqueakDatabaseError
from squeaknode.db.migrations import run_migrations
from squeaknode.db.models import Models


MAX_INT = 999999999999
MAX_HASH = b'\xff' * 32
INIT_NUM_RETRIES = 10
INIT_RETRY_INTERVAL_S = 1


logger = logging.getLogger(__name__)


class SqueakDb:
    def __init__(self, engine, schema=None):
        self.engine = engine
        self.schema = schema
        self.models = Models(schema=schema)

    @contextmanager
    def get_connection(self):
        with self.engine.connect() as connection:
            yield connection

    def init(self):
        """ Create the tables and indices in the database. """
        logger.debug("SqlAlchemy version: {}".format(sqlalchemy.__version__))
        run_migrations(self.engine)

        # Create aliases for profiles
        self.author_profiles = self.profiles.alias()

        # Create aliases for squeaks
        self.display_squeaks = self.squeaks.alias()

    def init_with_retries(
            self,
            num_retries=INIT_NUM_RETRIES,
            retry_interval_s=INIT_RETRY_INTERVAL_S,
    ):
        """ Try repeatedly to init the database.

        Raises exception if db init fails more than `num_retries` times.
        """
        n = 0
        while True:
            try:
                self.init()
                return
            except Exception:
                logger.exception("Failed to initialize database.")
                n += 1
                if n >= num_retries:
                    raise SqueakDatabaseError("Failed to initialize database.")
                time.sleep(retry_interval_s)

    @property
    def squeaks(self):
        return self.models.squeaks

    @property
    def profiles(self):
        return self.models.profiles

    @property
    def peers(self):
        return self.models.peers

    @property
    def received_offers(self):
        return self.models.received_offers

    @property
    def sent_payments(self):
        return self.models.sent_payments

    @property
    def received_payments(self):
        return self.models.received_payments

    @property
    def sent_offers(self):
        return self.models.sent_offers

    @property
    def users(self):
        return self.models.users

    @property
    def configs(self):
        return self.models.configs

    @property
    def twitter_accounts(self):
        return self.models.twitter_accounts

    @property
    def squeak_has_secret_key(self):
        return self.squeaks.c.secret_key != None  # noqa: E711

    @property
    def squeak_is_liked(self):
        return self.squeaks.c.liked_time_ms != None  # noqa: E711

    def squeak_is_older_than_retention(self, interval_s):
        return self.timestamp_now_ms > \
            self.squeaks.c.created_time_ms + interval_s * 1000

    def profile_has_private_key(self, profiles_table):
        return profiles_table.c.private_key != None  # noqa: E711

    def profile_is_following(self, profiles_table):
        return profiles_table.c.following == True  # noqa: E711

    @property
    def timestamp_now_ms(self):
        return int(time.time() * 1000)

    @property
    def received_offer_invoice_is_expired(self):
        expire_time = (
            self.received_offers.c.invoice_timestamp
            + self.received_offers.c.invoice_expiry
        )
        return self.timestamp_now_ms / 1000 >= expire_time

    def received_offer_is_out_of_retention(self, interval_s):
        expire_time_s = (
            self.received_offers.c.created_time_ms / 1000
            + interval_s
        )
        return self.timestamp_now_ms / 1000 >= expire_time_s

    @property
    def received_offer_is_paid(self):
        return self.received_offers.c.paid == True  # noqa: E711

    def sent_offer_out_of_retention(self, interval_s):
        expire_time = (
            self.sent_offers.c.invoice_timestamp
            + self.sent_offers.c.invoice_expiry
            + interval_s
        )
        return self.timestamp_now_ms / 1000 >= expire_time

    @property
    def sent_offer_is_paid(self):
        return self.sent_offers.c.paid == True  # noqa: E711

    @property
    def sent_offer_is_expired(self):
        expire_time = (
            self.sent_offers.c.invoice_timestamp
            + self.sent_offers.c.invoice_expiry
        )
        return self.timestamp_now_ms / 1000 >= expire_time

    def insert_squeak(self, squeak: CSqueak, block_header: CBlockHeader) -> Optional[bytes]:
        """ Insert a new squeak.

        Return the hash (bytes) of the inserted squeak.
        Return None if squeak already exists.
        """
        ins = self.squeaks.insert().values(
            created_time_ms=self.timestamp_now_ms,
            hash=get_hash(squeak),
            squeak=squeak.serialize(),
            block_hash=squeak.hashBlock,
            block_height=squeak.nBlockHeight,
            time_s=squeak.nTime,
            author_public_key=squeak.GetPubKey().to_bytes(),
            secret_key=None,
            block_time_s=block_header.nTime,
        )
        with self.get_connection() as connection:
            try:
                res = connection.execute(ins)
                squeak_hash = res.inserted_primary_key[0]
                return squeak_hash
            except sqlalchemy.exc.IntegrityError:
                logger.debug("Failed to insert squeak.", exc_info=True)
                return None

    def get_squeak(self, squeak_hash: bytes) -> Optional[CBaseSqueak]:
        """ Get a squeak. """
        s = select([self.squeaks]).where(
            self.squeaks.c.hash == squeak_hash)
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            return self._parse_squeak(row)

    def get_squeak_secret_key(self, squeak_hash: bytes) -> Optional[bytes]:
        """ Get a squeak secret key. """
        s = select([self.squeaks]).where(
            self.squeaks.c.hash == squeak_hash)
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            return row["secret_key"]

    def get_squeak_entry(self, squeak_hash: bytes) -> Optional[SqueakEntry]:
        """ Get a squeak with the author profile. """
        # author_profiles = self.profiles.alias()

        s = (
            select([
                self.squeaks,
                self.author_profiles,
            ])
            .select_from(
                self.squeaks
                .outerjoin(
                    self.author_profiles,
                    self.author_profiles.c.public_key == self.squeaks.c.author_public_key,
                )
            )
            .group_by(
                self.squeaks,
                self.author_profiles,
            )
            .where(self.squeaks.c.hash == squeak_hash)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            return self._parse_squeak_entry(
                row,
                # author_profiles_table=author_profiles,
            )

    def get_timeline_squeak_entries(
            self,
            limit: int,
            last_entry: Optional[SqueakEntry],
    ) -> List[SqueakEntry]:
        """ Get all followed squeaks. """
        last_block_height = last_entry.block_height if last_entry else MAX_INT
        last_squeak_time = last_entry.squeak_time if last_entry else MAX_INT
        last_squeak_hash = last_entry.squeak_hash if last_entry else MAX_HASH
        logger.info("""Timeline db query with
        limit: {}
        block_height: {}
        squeak_time: {}
        squeak_hash: {}
        """.format(
            limit,
            last_block_height,
            last_squeak_time,
            last_squeak_hash.hex(),
        ))
        s = (
            # select([self.squeaks, self.profiles])
            select([
                self.squeaks,
                self.author_profiles,
            ])
            .select_from(
                # self.squeaks.outerjoin(
                #     self.profiles,
                #     self.profiles.c.public_key == self.squeaks.c.author_public_key,
                # )
                self.squeaks
                .outerjoin(
                    self.author_profiles,
                    self.author_profiles.c.public_key == self.squeaks.c.author_public_key,
                )
            )
            .group_by(
                self.squeaks,
                self.author_profiles,
            )
            .where(self.profile_is_following(self.author_profiles))
            .where(
                tuple_(
                    self.squeaks.c.block_height,
                    self.squeaks.c.time_s,
                    self.squeaks.c.hash,
                ) < tuple_(
                    last_block_height,
                    last_squeak_time,
                    last_squeak_hash,
                )
            )
            .order_by(
                self.squeaks.c.block_height.desc(),
                self.squeaks.c.time_s.desc(),
                self.squeaks.c.hash.desc(),
            )
            .limit(limit)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            return [self._parse_squeak_entry(row) for row in rows]

    def get_squeak_entries_for_public_key(
            self,
            public_key: SqueakPublicKey,
            limit: int,
            last_entry: Optional[SqueakEntry],
    ) -> List[SqueakEntry]:
        """ Get a squeak. """
        last_block_height = last_entry.block_height if last_entry else MAX_INT
        last_squeak_time = last_entry.squeak_time if last_entry else MAX_INT
        last_squeak_hash = last_entry.squeak_hash if last_entry else MAX_HASH
        logger.info("""Address db query with
        public key: {}
        limit: {}
        block_height: {}
        squeak_time: {}
        squeak_hash: {}
        """.format(
            public_key,
            limit,
            last_block_height,
            last_squeak_time,
            last_squeak_hash.hex(),
        ))
        s = (
            # select([self.squeaks, self.profiles])
            select([
                self.squeaks,
                self.author_profiles,
            ])
            .select_from(
                # self.squeaks.outerjoin(
                #     self.profiles,
                #     self.profiles.c.public_key == self.squeaks.c.author_public_key,
                # )
                self.squeaks
                .outerjoin(
                    self.author_profiles,
                    self.author_profiles.c.public_key == self.squeaks.c.author_public_key,
                )
            )
            .group_by(
                self.squeaks,
                self.author_profiles,
            )
            .where(self.squeaks.c.author_public_key == public_key.to_bytes())
            .where(
                tuple_(
                    self.squeaks.c.block_height,
                    self.squeaks.c.time_s,
                    self.squeaks.c.hash,
                ) < tuple_(
                    last_block_height,
                    last_squeak_time,
                    last_squeak_hash,
                )
            )
            .order_by(
                self.squeaks.c.block_height.desc(),
                self.squeaks.c.time_s.desc(),
                self.squeaks.c.hash.desc(),
            )
            .limit(limit)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            return [self._parse_squeak_entry(row) for row in rows]

    def get_squeak_entries_for_text_search(
            self,
            search_text: str,
            limit: int,
            last_entry: Optional[SqueakEntry],
    ) -> List[SqueakEntry]:
        """ Get a squeak. """
        last_block_height = last_entry.block_height if last_entry else MAX_INT
        last_squeak_time = last_entry.squeak_time if last_entry else MAX_INT
        last_squeak_hash = last_entry.squeak_hash if last_entry else MAX_HASH
        logger.info("""Search db query with
        search_text: {}
        limit: {}
        block_height: {}
        squeak_time: {}
        squeak_hash: {}
        """.format(
            search_text,
            limit,
            last_block_height,
            last_squeak_time,
            last_squeak_hash.hex(),
        ))
        s = (
            # select([self.squeaks, self.profiles])
            select([
                self.squeaks,
                self.author_profiles,
            ])
            .select_from(
                # self.squeaks.outerjoin(
                #     self.profiles,
                #     self.profiles.c.public_key == self.squeaks.c.author_public_key,
                # )
                self.squeaks
                .outerjoin(
                    self.author_profiles,
                    self.author_profiles.c.public_key == self.squeaks.c.author_public_key,
                )
            )
            .group_by(
                self.squeaks,
                self.author_profiles,
            )
            .where(self.squeaks.c.content.ilike(f'%{search_text}%'))
            .where(
                tuple_(
                    self.squeaks.c.block_height,
                    self.squeaks.c.time_s,
                    self.squeaks.c.hash,
                ) < tuple_(
                    last_block_height,
                    last_squeak_time,
                    last_squeak_hash,
                )
            )
            .order_by(
                self.squeaks.c.block_height.desc(),
                self.squeaks.c.time_s.desc(),
                self.squeaks.c.hash.desc(),
            )
            .limit(limit)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            return [self._parse_squeak_entry(row) for row in rows]

    def get_number_of_squeaks(self) -> int:
        """ Get total number of squeaks. """
        s = (
            select([
                func.count().label("num_squeaks"),
            ])
            .select_from(self.squeaks)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            num_squeaks = row["num_squeaks"]
            return num_squeaks

    def number_of_squeaks_with_public_key_with_block_height(
        self,
        public_key: SqueakPublicKey,
        block_height: int,
    ) -> int:
        """ Get number of squeaks with public key with block height. """
        s = (
            select([
                func.count().label("num_squeaks"),
            ])
            .select_from(self.squeaks)
            .where(self.squeaks.c.author_public_key == public_key.to_bytes())
            .where(self.squeaks.c.block_height == block_height)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            num_squeaks = row["num_squeaks"]
            return num_squeaks

    def get_old_squeaks_to_delete(
            self,
            interval_s: int,
    ) -> List[bytes]:
        """ Get squeaks older than retention that meet the
        criteria for deletion.
        """
        s = (
            # select([self.squeaks, self.profiles])
            select([self.squeaks, self.author_profiles])
            .select_from(
                # self.squeaks.outerjoin(
                #     self.profiles,
                #     self.profiles.c.public_key == self.squeaks.c.author_public_key,
                # )
                self.squeaks
                .outerjoin(
                    self.author_profiles,
                    self.author_profiles.c.public_key == self.squeaks.c.author_public_key,
                )
            )
            .where(self.squeak_is_older_than_retention(interval_s))
            .where(not_(self.profile_has_private_key(self.author_profiles)))
            .where(not_(self.squeak_is_liked))
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            hashes = [(row["hash"]) for row in rows]
            return hashes

    def insert_profile(self, squeak_profile: SqueakProfile) -> int:
        """ Insert a new squeak profile. """
        ins = self.profiles.insert().values(
            created_time_ms=self.timestamp_now_ms,
            profile_name=squeak_profile.profile_name,
            private_key=squeak_profile.private_key.to_bytes(
            ) if squeak_profile.private_key else None,
            public_key=squeak_profile.public_key.to_bytes(),
            following=squeak_profile.following,
        )
        with self.get_connection() as connection:
            res = connection.execute(ins)
            profile_id = res.inserted_primary_key[0]
            return profile_id

    def get_profiles(self) -> List[SqueakProfile]:
        """ Get all profiles. """
        s = select([self.profiles])
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            profiles = [self._parse_squeak_profile(row) for row in rows]
            return profiles

    def get_signing_profiles(self) -> List[SqueakProfile]:
        """ Get all signing profiles. """
        s = (
            select([self.profiles])
            .where(self.profile_has_private_key(self.profiles))
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            profiles = [self._parse_squeak_profile(row) for row in rows]
            return profiles

    def get_contact_profiles(self) -> List[SqueakProfile]:
        """ Get all contact profiles. """
        s = (
            select([self.profiles])
            .where(not_(self.profile_has_private_key(self.profiles)))
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            profiles = [self._parse_squeak_profile(row) for row in rows]
            return profiles

    def get_following_profiles(self) -> List[SqueakProfile]:
        """ Get all following profiles. """
        s = select([self.profiles]).where(self.profiles.c.following)
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            profiles = [self._parse_squeak_profile(row) for row in rows]
            return profiles

    def get_profile(self, profile_id: int) -> Optional[SqueakProfile]:
        """ Get a profile. """
        s = select([self.profiles]).where(
            self.profiles.c.profile_id == profile_id)
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            return self._parse_squeak_profile(row)

    def get_profile_by_public_key(self, public_key: SqueakPublicKey) -> Optional[SqueakProfile]:
        """ Get a profile by public key. """
        s = select([self.profiles]).where(
            self.profiles.c.public_key == public_key.to_bytes())
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            return self._parse_squeak_profile(row)

    def get_profile_by_name(self, name: str) -> Optional[SqueakProfile]:
        """ Get a profile by name. """
        s = select([self.profiles]).where(self.profiles.c.profile_name == name)
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            return self._parse_squeak_profile(row)

    def set_profile_following(self, profile_id: int, following: bool) -> None:
        """ Set a profile is following. """
        stmt = (
            self.profiles.update()
            .where(self.profiles.c.profile_id == profile_id)
            .values(following=following)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def set_profile_name(self, profile_id: int, profile_name: str) -> None:
        """ Set a profile name. """
        stmt = (
            self.profiles.update()
            .where(self.profiles.c.profile_id == profile_id)
            .values(profile_name=profile_name)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def delete_profile(self, profile_id: int) -> None:
        """ Delete a profile. """
        delete_profile_stmt = self.profiles.delete().where(
            self.profiles.c.profile_id == profile_id
        )
        with self.get_connection() as connection:
            connection.execute(delete_profile_stmt)

    def set_profile_image(self, profile_id: int, profile_image: Optional[bytes]) -> None:
        """ Set a profile image. """
        stmt = (
            self.profiles.update()
            .where(self.profiles.c.profile_id == profile_id)
            .values(profile_image=profile_image)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def set_squeak_secret_key(self, squeak_hash: bytes, secret_key: bytes) -> None:
        """ Set the secret key of a squeak. """
        stmt = (
            self.squeaks.update()
            .where(self.squeaks.c.hash == squeak_hash)
            .values(secret_key=secret_key)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def set_squeak_decrypted_content(self, squeak_hash: bytes, content: str) -> None:
        """ Set the decrypted content of a squeak. """
        stmt = (
            self.squeaks.update()
            .where(self.squeaks.c.hash == squeak_hash)
            .values(content=content)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def set_squeak_liked(self, squeak_hash: bytes) -> None:
        """ Set the squeak to be liked. """
        stmt = (
            self.squeaks.update()
            .where(self.squeaks.c.hash == squeak_hash)
            .values(liked_time_ms=self.timestamp_now_ms)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def set_squeak_unliked(self, squeak_hash: bytes) -> None:
        """ Set the squeak to be unliked. """
        stmt = (
            self.squeaks.update()
            .where(self.squeaks.c.hash == squeak_hash)
            .values(liked_time_ms=None)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def delete_squeak(self, squeak_hash: bytes) -> None:
        """ Delete a squeak. """
        delete_squeak_stmt = self.squeaks.delete().where(
            self.squeaks.c.hash == squeak_hash
        )
        with self.get_connection() as connection:
            connection.execute(delete_squeak_stmt)

    def insert_peer(self, squeak_peer: SqueakPeer) -> int:
        """ Insert a new squeak peer. """
        ins = self.peers.insert().values(
            created_time_ms=self.timestamp_now_ms,
            peer_name=squeak_peer.peer_name,
            network=squeak_peer.address.network.name,
            host=squeak_peer.address.host,
            port=squeak_peer.address.port,
            autoconnect=squeak_peer.autoconnect,
            share_for_free=squeak_peer.share_for_free,
        )
        with self.get_connection() as connection:
            res = connection.execute(ins)
            id = res.inserted_primary_key[0]
            return id

    def get_peer(self, peer_id: int) -> Optional[SqueakPeer]:
        """ Get a peer. """
        s = select([self.peers]).where(self.peers.c.peer_id == peer_id)
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            return self._parse_squeak_peer(row)

    def get_peer_by_address(self, peer_address: PeerAddress) -> Optional[SqueakPeer]:
        """ Get a peer by address. """
        s = (
            select([self.peers])
            .where(self.peers.c.host == peer_address.host)
            .where(self.peers.c.port == peer_address.port)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            return self._parse_squeak_peer(row)

    def get_peers(self) -> List[SqueakPeer]:
        """ Get all peers. """
        s = select([self.peers])
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            peers = [self._parse_squeak_peer(row) for row in rows]
            return peers

    def get_autoconnect_peers(self) -> List[SqueakPeer]:
        """ Get peers that are set to be autoconnect. """
        s = select([self.peers]).where(self.peers.c.autoconnect)
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            peers = [self._parse_squeak_peer(row) for row in rows]
            return peers

    def set_peer_autoconnect(self, peer_id: int, autoconnect: bool):
        """ Set a peer is autoconnect. """
        stmt = (
            self.peers.update()
            .where(self.peers.c.peer_id == peer_id)
            .values(autoconnect=autoconnect)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def set_peer_share_for_free(self, peer_id: int, share_for_free: bool):
        """ Set a peer is share_for_free. """
        stmt = (
            self.peers.update()
            .where(self.peers.c.peer_id == peer_id)
            .values(share_for_free=share_for_free)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def set_peer_name(self, peer_id: int, peer_name: str):
        """ Set a peer name. """
        stmt = (
            self.peers.update()
            .where(self.peers.c.peer_id == peer_id)
            .values(peer_name=peer_name)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def delete_peer(self, peer_id: int):
        """ Delete a peer. """
        delete_peer_stmt = self.peers.delete().where(self.peers.c.peer_id == peer_id)
        with self.get_connection() as connection:
            connection.execute(delete_peer_stmt)

    def insert_received_offer(self, received_offer: ReceivedOffer) -> Optional[int]:
        """ Insert a new received offer.

        Return the received offer id of the inserted received offer.
        Return None if received offer already exists.
        """
        ins = self.received_offers.insert().values(
            created_time_ms=self.timestamp_now_ms,
            squeak_hash=received_offer.squeak_hash,
            payment_hash=received_offer.payment_hash,
            nonce=received_offer.nonce,
            payment_point=received_offer.payment_point,
            invoice_timestamp=received_offer.invoice_timestamp,
            invoice_expiry=received_offer.invoice_expiry,
            price_msat=received_offer.price_msat,
            payment_request=received_offer.payment_request,
            destination=received_offer.destination,
            lightning_host=received_offer.lightning_address.host,
            lightning_port=received_offer.lightning_address.port,
            peer_network=received_offer.peer_address.network.name,
            peer_host=received_offer.peer_address.host,
            peer_port=received_offer.peer_address.port,
        )
        with self.get_connection() as connection:
            try:
                res = connection.execute(ins)
                received_offer_id = res.inserted_primary_key[0]
                return received_offer_id
            except sqlalchemy.exc.IntegrityError:
                logger.debug("Failed to insert received offer.", exc_info=True)
                return None

    def get_received_offers(self, squeak_hash: bytes) -> List[ReceivedOffer]:
        """ Get offers with peer for a squeak hash. """
        s = (
            select([self.received_offers])
            .where(self.received_offers.c.squeak_hash == squeak_hash)
            .where(not_(self.received_offer_is_paid))
            .where(not_(self.received_offer_invoice_is_expired))
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            offers = [
                self._parse_received_offer(row)
                for row in rows
            ]
            return offers

    def get_received_offer(self, received_offer_id: int) -> Optional[ReceivedOffer]:
        """ Get offer with peer for an offer id. """
        s = (
            select([self.received_offers])
            .where(self.received_offers.c.received_offer_id == received_offer_id)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            offer = self._parse_received_offer(row)
            return offer

    def delete_expired_received_offers(self, interval_s):
        """ Delete all expired offers. """
        s = self.received_offers.delete().where(
            or_(
                self.received_offer_invoice_is_expired,
                self.received_offer_is_out_of_retention(interval_s),
            )
        )
        with self.get_connection() as connection:
            res = connection.execute(s)
            deleted_offers = res.rowcount
            return deleted_offers

    def set_received_offer_paid(self, payment_hash: bytes, paid: bool) -> None:
        """ Set a received offer is paid. """
        stmt = (
            self.received_offers.update()
            .where(self.received_offers.c.payment_hash == payment_hash)
            .values(paid=paid)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def insert_sent_payment(self, sent_payment: SentPayment) -> int:
        """ Insert a new sent payment. """
        ins = self.sent_payments.insert().values(
            created_time_ms=self.timestamp_now_ms,
            peer_network=sent_payment.peer_address.network.name,
            peer_host=sent_payment.peer_address.host,
            peer_port=sent_payment.peer_address.port,
            squeak_hash=sent_payment.squeak_hash,
            payment_hash=sent_payment.payment_hash,
            secret_key=sent_payment.secret_key,
            price_msat=sent_payment.price_msat,
            node_pubkey=sent_payment.node_pubkey,
            valid=sent_payment.valid,
        )
        with self.get_connection() as connection:
            res = connection.execute(ins)
            sent_payment_id = res.inserted_primary_key[0]
            return sent_payment_id

    def get_sent_payments(
            self,
            limit: int,
            last_sent_payment: Optional[SentPayment],
    ) -> List[SentPayment]:
        """ Get all sent payments. """
        last_created_time = last_sent_payment.created_time_ms if last_sent_payment else self.timestamp_now_ms
        last_payment_hash = last_sent_payment.payment_hash if last_sent_payment else MAX_HASH
        logger.info("""Get sent payments db query with
        limit: {}
        last_created_time: {}
        last_payment_hash: {}
        """.format(
            limit,
            last_created_time,
            last_payment_hash.hex(),
        ))
        s = (
            select([self.sent_payments])
            .where(
                tuple_(
                    self.sent_payments.c.created_time_ms,
                    self.sent_payments.c.payment_hash,
                ) < tuple_(
                    last_created_time,
                    last_payment_hash,
                )
            )
            .order_by(
                self.sent_payments.c.created_time_ms.desc(),
                self.sent_payments.c.payment_hash.desc(),
            )
            .limit(limit)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            sent_payments = [
                self._parse_sent_payment(row) for row in rows]
            return sent_payments

    def get_sent_payments_for_squeak(
            self,
            squeak_hash: bytes,
            limit: int,
            last_sent_payment: Optional[SentPayment],
    ) -> List[SentPayment]:
        """ Get sent payments for a squeak. """
        last_created_time = last_sent_payment.created_time_ms if last_sent_payment else self.timestamp_now_ms
        last_payment_hash = last_sent_payment.payment_hash if last_sent_payment else MAX_HASH
        logger.info("""Get sent payments db query with
        limit: {}
        last_created_time: {}
        last_payment_hash: {}
        """.format(
            limit,
            last_created_time,
            last_payment_hash.hex(),
        ))
        s = (
            select([self.sent_payments])
            .where(self.sent_payments.c.squeak_hash == squeak_hash)
            .where(
                tuple_(
                    self.sent_payments.c.created_time_ms,
                    self.sent_payments.c.payment_hash,
                ) < tuple_(
                    last_created_time,
                    last_payment_hash,
                )
            )
            .order_by(
                self.sent_payments.c.created_time_ms.desc(),
                self.sent_payments.c.payment_hash.desc(),
            )
            .limit(limit)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            sent_payments = [
                self._parse_sent_payment(row) for row in rows]
            return sent_payments

    def get_sent_payments_for_pubkey(
            self,
            public_key: SqueakPublicKey,
            limit: int,
            last_sent_payment: Optional[SentPayment],
    ) -> List[SentPayment]:
        """ Get sent payments for a pubkey. """
        last_created_time = last_sent_payment.created_time_ms if last_sent_payment else self.timestamp_now_ms
        last_payment_hash = last_sent_payment.payment_hash if last_sent_payment else MAX_HASH
        logger.info("""Get sent payments db query with
        public_key: {}
        limit: {}
        last_created_time: {}
        last_payment_hash: {}
        """.format(
            public_key,
            limit,
            last_created_time,
            last_payment_hash.hex(),
        ))
        s = (
            select([self.sent_payments, self.squeaks])
            .select_from(
                self.sent_payments.outerjoin(
                    self.squeaks,
                    self.sent_payments.c.squeak_hash == self.squeaks.c.hash,
                )
            )
            .where(self.squeaks.c.author_public_key == public_key.to_bytes())
            .where(
                tuple_(
                    self.sent_payments.c.created_time_ms,
                    self.sent_payments.c.payment_hash,
                ) < tuple_(
                    last_created_time,
                    last_payment_hash,
                )
            )
            .order_by(
                self.sent_payments.c.created_time_ms.desc(),
                self.sent_payments.c.payment_hash.desc(),
            )
            .limit(limit)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            sent_payments = [
                self._parse_sent_payment(row) for row in rows]
            return sent_payments

    def get_sent_payments_for_peer(
            self,
            peer_address: PeerAddress,
            limit: int,
            last_sent_payment: Optional[SentPayment],
    ) -> List[SentPayment]:
        """ Get sent payments for a peer. """
        last_created_time = last_sent_payment.created_time_ms if last_sent_payment else self.timestamp_now_ms
        last_payment_hash = last_sent_payment.payment_hash if last_sent_payment else MAX_HASH
        logger.info("""Get sent payments db query with
        peer_address: {}
        limit: {}
        last_created_time: {}
        last_payment_hash: {}
        """.format(
            peer_address,
            limit,
            last_created_time,
            last_payment_hash.hex(),
        ))
        s = (
            select([self.sent_payments])
            .where(self.received_payments.c.peer_network == peer_address.network.name)
            .where(self.received_payments.c.peer_host == peer_address.host)
            .where(self.received_payments.c.peer_port == peer_address.port)
            .where(
                tuple_(
                    self.sent_payments.c.created_time_ms,
                    self.sent_payments.c.payment_hash,
                ) < tuple_(
                    last_created_time,
                    last_payment_hash,
                )
            )
            .order_by(
                self.sent_payments.c.created_time_ms.desc(),
                self.sent_payments.c.payment_hash.desc(),
            )
            .limit(limit)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            sent_payments = [
                self._parse_sent_payment(row) for row in rows]
            return sent_payments

    def get_sent_payment(self, sent_payment_id: int) -> Optional[SentPayment]:
        """ Get sent payment by id. """
        s = (
            select([self.sent_payments])
            .where(self.sent_payments.c.sent_payment_id == sent_payment_id)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            return self._parse_sent_payment(row)

    def insert_sent_offer(self, sent_offer: SentOffer):
        """ Insert a new sent offer. """
        ins = self.sent_offers.insert().values(
            created_time_ms=self.timestamp_now_ms,
            squeak_hash=sent_offer.squeak_hash,
            payment_hash=sent_offer.payment_hash,
            nonce=sent_offer.nonce,
            price_msat=sent_offer.price_msat,
            payment_request=sent_offer.payment_request,
            invoice_timestamp=sent_offer.invoice_time,
            invoice_expiry=sent_offer.invoice_expiry,
            peer_network=sent_offer.peer_address.network.name,
            peer_host=sent_offer.peer_address.host,
            peer_port=sent_offer.peer_address.port,
        )
        with self.get_connection() as connection:
            res = connection.execute(ins)
            sent_offer_id = res.inserted_primary_key[0]
            return sent_offer_id

    def get_sent_offers(self) -> List[SentOffer]:
        """ Get all received payments. """
        s = select([self.sent_offers]).order_by(
            self.sent_offers.c.created_time_ms.desc(),
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            sent_offers = [self._parse_sent_offer(row) for row in rows]
            return sent_offers

    def get_sent_offer_by_payment_hash(self, payment_hash: bytes) -> Optional[SentOffer]:
        """ Get a sent offer by preimage hash. """
        s = select([self.sent_offers]).where(
            self.sent_offers.c.payment_hash == payment_hash
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            sent_offer = self._parse_sent_offer(row)
            return sent_offer

    def get_sent_offer_by_squeak_hash_and_peer(self, squeak_hash: bytes, peer_address: PeerAddress) -> Optional[SentOffer]:
        """
        Get a sent offer by squeak hash and peer address host. Only
        return sent offer if it's not expired and not paid.

        TODO: add where clause for peer address network.
        """
        s = (
            select([self.sent_offers])
            .where(self.sent_offers.c.squeak_hash == squeak_hash)
            .where(self.sent_offers.c.peer_network == peer_address.network.name)
            .where(self.sent_offers.c.peer_host == peer_address.host)
            .where(not_(self.sent_offer_is_paid))
            .where(not_(self.sent_offer_is_expired))
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            sent_offer = self._parse_sent_offer(row)
            return sent_offer

    def delete_expired_sent_offers(self, interval_s):
        """
        Delete all expired sent offers. Only delete sent
        offers that have been expired for more than interval_s seconds.
        """
        s = self.sent_offers.delete().where(
            self.sent_offer_out_of_retention(interval_s)
        )
        with self.get_connection() as connection:
            res = connection.execute(s)
            deleted_sent_offers = res.rowcount
            return deleted_sent_offers

    def set_sent_offer_paid(self, payment_hash: bytes, paid: bool) -> None:
        """ Set a sent offer is paid. """
        stmt = (
            self.sent_offers.update()
            .where(self.sent_offers.c.payment_hash == payment_hash)
            .values(paid=paid)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def insert_received_payment(self, received_payment: ReceivedPayment) -> Optional[int]:
        """ Insert a new received payment.

        Return the received payment id of the inserted received payment.
        Return None if received payment already exists.
        """
        ins = self.received_payments.insert().values(
            created_time_ms=self.timestamp_now_ms,
            squeak_hash=received_payment.squeak_hash,
            payment_hash=received_payment.payment_hash,
            price_msat=received_payment.price_msat,
            settle_index=received_payment.settle_index,
            peer_network=received_payment.peer_address.network.name,
            peer_host=received_payment.peer_address.host,
            peer_port=received_payment.peer_address.port,
        )
        with self.get_connection() as connection:
            try:
                res = connection.execute(ins)
                received_payment_id = res.inserted_primary_key[0]
                return received_payment_id
            except sqlalchemy.exc.IntegrityError:
                logger.debug(
                    "Failed to insert received payment.", exc_info=True)
                return None

    def get_received_payments(
            self,
            limit: int,
            last_received_payment: Optional[ReceivedPayment],
    ) -> List[ReceivedPayment]:
        """ Get all received payments. """
        last_created_time = last_received_payment.created_time_ms if last_received_payment else self.timestamp_now_ms
        last_payment_hash = last_received_payment.payment_hash if last_received_payment else MAX_HASH
        logger.info("""Get received payments db query with
        limit: {}
        last_created_time: {}
        last_payment_hash: {}
        """.format(
            limit,
            last_created_time,
            last_payment_hash.hex(),
        ))
        s = (
            select([self.received_payments])
            .where(
                tuple_(
                    self.received_payments.c.created_time_ms,
                    self.received_payments.c.payment_hash,
                ) < tuple_(
                    last_created_time,
                    last_payment_hash,
                )
            )
            .order_by(
                self.received_payments.c.created_time_ms.desc(),
                self.received_payments.c.payment_hash.desc(),
            )
            .limit(limit)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            received_payments = [
                self._parse_received_payment(row) for row in rows]
            return received_payments

    def get_received_payments_for_squeak(
            self,
            squeak_hash: bytes,
            limit: int,
            last_received_payment: Optional[ReceivedPayment],
    ) -> List[ReceivedPayment]:
        """ Get received payments for a squeak. """
        last_created_time = last_received_payment.created_time_ms if last_received_payment else self.timestamp_now_ms
        last_payment_hash = last_received_payment.payment_hash if last_received_payment else MAX_HASH
        logger.info("""Get received payments db query with
        limit: {}
        last_created_time: {}
        last_payment_hash: {}
        """.format(
            limit,
            last_created_time,
            last_payment_hash.hex(),
        ))
        s = (
            select([self.received_payments])
            .where(self.received_payments.c.squeak_hash == squeak_hash)
            .where(
                tuple_(
                    self.received_payments.c.created_time_ms,
                    self.received_payments.c.payment_hash,
                ) < tuple_(
                    last_created_time,
                    last_payment_hash,
                )
            )
            .order_by(
                self.received_payments.c.created_time_ms.desc(),
                self.received_payments.c.payment_hash.desc(),
            )
            .limit(limit)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            received_payments = [
                self._parse_received_payment(row) for row in rows]
            return received_payments

    def get_received_payments_for_pubkey(
            self,
            public_key: SqueakPublicKey,
            limit: int,
            last_received_payment: Optional[ReceivedPayment],
    ) -> List[ReceivedPayment]:
        """ Get received payments for a pubkey. """
        last_created_time = last_received_payment.created_time_ms if last_received_payment else self.timestamp_now_ms
        last_payment_hash = last_received_payment.payment_hash if last_received_payment else MAX_HASH
        logger.info("""Get received payments db query with
        limit: {}
        last_created_time: {}
        last_payment_hash: {}
        """.format(
            limit,
            last_created_time,
            last_payment_hash.hex(),
        ))
        s = (
            select([self.received_payments, self.squeaks])
            .select_from(
                self.received_payments.outerjoin(
                    self.squeaks,
                    self.received_payments.c.squeak_hash == self.squeaks.c.hash,
                )
            )
            .where(self.squeaks.c.author_public_key == public_key.to_bytes())
            .where(
                tuple_(
                    self.received_payments.c.created_time_ms,
                    self.received_payments.c.payment_hash,
                ) < tuple_(
                    last_created_time,
                    last_payment_hash,
                )
            )
            .order_by(
                self.received_payments.c.created_time_ms.desc(),
                self.received_payments.c.payment_hash.desc(),
            )
            .limit(limit)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            received_payments = [
                self._parse_received_payment(row) for row in rows]
            return received_payments

    def get_received_payments_for_peer(
            self,
            peer_address: PeerAddress,
            limit: int,
            last_received_payment: Optional[ReceivedPayment],
    ) -> List[ReceivedPayment]:
        """ Get received payments for a squeak. """
        last_created_time = last_received_payment.created_time_ms if last_received_payment else self.timestamp_now_ms
        last_payment_hash = last_received_payment.payment_hash if last_received_payment else MAX_HASH
        logger.info("""Get received payments db query with
        peer_address: {}
        limit: {}
        last_created_time: {}
        last_payment_hash: {}
        """.format(
            peer_address,
            limit,
            last_created_time,
            last_payment_hash.hex(),
        ))
        s = (
            select([self.received_payments])
            .where(self.received_payments.c.peer_network == peer_address.network.name)
            .where(self.received_payments.c.peer_host == peer_address.host)
            .where(self.received_payments.c.peer_port == peer_address.port)
            .where(
                tuple_(
                    self.received_payments.c.created_time_ms,
                    self.received_payments.c.payment_hash,
                ) < tuple_(
                    last_created_time,
                    last_payment_hash,
                )
            )
            .order_by(
                self.received_payments.c.created_time_ms.desc(),
                self.received_payments.c.payment_hash.desc(),
            )
            .limit(limit)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            received_payments = [
                self._parse_received_payment(row) for row in rows]
            return received_payments

    def get_latest_settle_index(self) -> Optional[int]:
        """ Get the lnd settled index of the most recent received payment. """
        s = select(
            [func.max(self.received_payments.c.settle_index)],
        ).select_from(self.received_payments)
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            latest_index = row[0]
            return latest_index

    def clear_received_payment_settle_indices(self) -> None:
        """ Set settle_index to zero for all received payments. """
        stmt = (
            self.received_payments.update()
            .values(settle_index=0)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def yield_received_payments_from_index(self, start_index: int = 0) -> Iterator[ReceivedPayment]:
        """ Get all received payments. """
        s = (
            select([self.received_payments])
            .order_by(
                self.received_payments.c.received_payment_id.asc(),
            )
            .where(self.received_payments.c.received_payment_id > start_index)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            for row in result:
                received_payment = self._parse_received_payment(row)
                yield received_payment

    def get_received_payment_summary(self) -> ReceivedPaymentSummary:
        """ Get received payment summary. """
        s = select([
            func.count().label("num_payments_received"),
            func.sum(self.received_payments.c.price_msat).label(
                "total_amount_received_msat"),
        ]).select_from(self.received_payments)
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            received_payment_summary = self._parse_received_payment_summary(
                row)
            return received_payment_summary

    def get_sent_payment_summary(self) -> SentPaymentSummary:
        """ Get sent payment summary. """
        s = select([
            func.count().label("num_payments_sent"),
            func.sum(self.sent_payments.c.price_msat).label(
                "total_amount_sent_msat"),
        ]).select_from(self.sent_payments)
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            sent_payment_summary = self._parse_sent_payment_summary(row)
            return sent_payment_summary

    def get_received_payment_summary_for_squeak(self, squeak_hash: bytes) -> ReceivedPaymentSummary:
        """ Get received payment summary for a single squeak. """
        s = (
            select([
                func.count().label("num_payments_received"),
                func.sum(self.received_payments.c.price_msat).label(
                    "total_amount_received_msat"),
            ])
            .select_from(self.received_payments)
            .where(self.received_payments.c.squeak_hash == squeak_hash)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            received_payment_summary = self._parse_received_payment_summary(
                row)
            return received_payment_summary

    def get_sent_payment_summary_for_squeak(self, squeak_hash: bytes) -> SentPaymentSummary:
        """ Get sent payment summary for a squeak. """
        s = (
            select([
                func.count().label("num_payments_sent"),
                func.sum(self.sent_payments.c.price_msat).label(
                    "total_amount_sent_msat"),
            ])
            .select_from(self.sent_payments)
            .where(self.sent_payments.c.squeak_hash == squeak_hash)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            sent_payment_summary = self._parse_sent_payment_summary(row)
            return sent_payment_summary

    def get_received_payment_summary_for_pubkey(self, public_key: SqueakPublicKey) -> ReceivedPaymentSummary:
        """ Get received payment summary for a single pubkey. """
        s = (
            select([
                func.count().label("num_payments_received"),
                func.sum(self.received_payments.c.price_msat).label(
                    "total_amount_received_msat"),
            ])
            .select_from(
                self.received_payments.outerjoin(
                    self.squeaks,
                    self.received_payments.c.squeak_hash == self.squeaks.c.hash,
                )
            )
            .select_from(self.received_payments)
            .where(self.squeaks.c.author_public_key == public_key.to_bytes())
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            received_payment_summary = self._parse_received_payment_summary(
                row)
            return received_payment_summary

    def get_sent_payment_summary_for_pubkey(self, public_key: SqueakPublicKey) -> SentPaymentSummary:
        """ Get sent payment summary for a pubkey. """
        s = (
            select([
                func.count().label("num_payments_sent"),
                func.sum(self.sent_payments.c.price_msat).label(
                    "total_amount_sent_msat"),
            ])
            .select_from(
                self.sent_payments.outerjoin(
                    self.squeaks,
                    self.sent_payments.c.squeak_hash == self.squeaks.c.hash,
                )
            )
            .where(self.squeaks.c.author_public_key == public_key.to_bytes())
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            sent_payment_summary = self._parse_sent_payment_summary(row)
            return sent_payment_summary

    def get_received_payment_summary_for_peer(self, peer_address: PeerAddress) -> ReceivedPaymentSummary:
        """ Get received payment summary for a single peer. """
        s = (
            select([
                func.count().label("num_payments_received"),
                func.sum(self.received_payments.c.price_msat).label(
                    "total_amount_received_msat"),
            ])
            .select_from(self.received_payments)
            .where(self.received_payments.c.peer_network == peer_address.network.name)
            .where(self.received_payments.c.peer_host == peer_address.host)
            .where(self.received_payments.c.peer_port == peer_address.port)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            received_payment_summary = self._parse_received_payment_summary(
                row)
            return received_payment_summary

    def get_sent_payment_summary_for_peer(self, peer_address: PeerAddress) -> SentPaymentSummary:
        """ Get sent payment summary for a peer. """
        s = (
            select([
                func.count().label("num_payments_sent"),
                func.sum(self.sent_payments.c.price_msat).label(
                    "total_amount_sent_msat"),
            ])
            .select_from(self.sent_payments)
            .where(self.received_payments.c.peer_network == peer_address.network.name)
            .where(self.received_payments.c.peer_host == peer_address.host)
            .where(self.received_payments.c.peer_port == peer_address.port)
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            sent_payment_summary = self._parse_sent_payment_summary(row)
            return sent_payment_summary

    def insert_config(self, user_config: UserConfig) -> Optional[str]:
        """ Insert a new config.

        Return the name (str) of the inserted config user.
        Return None if config already exists.
        """
        ins = self.configs.insert().values(
            username=user_config.username,
        )
        with self.get_connection() as connection:
            try:
                res = connection.execute(ins)
                username = res.inserted_primary_key[0]
                return username
            except sqlalchemy.exc.IntegrityError:
                logger.debug("Failed to insert config.", exc_info=True)
                return None

    def get_config(self, username: str) -> Optional[UserConfig]:
        """ Get a config. """
        s = select([self.configs]).where(self.configs.c.username == username)
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            return self._parse_user_config(row)

    def set_config_sell_price_msat(self, username: str, sell_price_msat: int) -> None:
        """ Set a config sell price msat. """
        stmt = (
            self.configs.update()
            .where(self.configs.c.username == username)
            .values(sell_price_msat=sell_price_msat)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def clear_config_sell_price_msat(self, username: str) -> None:
        """ Clear a config sell price msat. """
        stmt = (
            self.configs.update()
            .where(self.configs.c.username == username)
            .values(sell_price_msat=None)
        )
        with self.get_connection() as connection:
            connection.execute(stmt)

    def insert_twitter_account(self, twitter_account: TwitterAccount) -> Optional[int]:
        """ Insert a new twitter account mapping to a squeak profile.

        Return the id (int) of the inserted twitter account.
        Return None if twitter account already exists.
        """
        ins = self.twitter_accounts.insert().values(
            handle=twitter_account.handle,
            profile_id=twitter_account.profile_id,
            bearer_token=twitter_account.bearer_token,
        )
        with self.get_connection() as connection:
            try:
                res = connection.execute(ins)
                twitter_account_id = res.inserted_primary_key[0]
                return twitter_account_id
            except sqlalchemy.exc.IntegrityError:
                logger.debug(
                    "Failed to insert twitter account.", exc_info=True)
                return None

    def get_twitter_accounts(self) -> List[TwitterAccountEntry]:
        """ Get all twitter accounts. """
        s = (
            select([self.twitter_accounts, self.profiles])
            .select_from(
                self.twitter_accounts.outerjoin(
                    self.profiles,
                    self.profiles.c.profile_id == self.twitter_accounts.c.profile_id,
                )
            )
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            twitter_accounts = [
                self._parse_twitter_account_entry(row) for row in rows]
            return twitter_accounts

    def delete_twitter_account(self, twitter_account_id: int) -> None:
        """ Delete a twitter_account. """
        delete_twitter_account_stmt = self.twitter_accounts.delete().where(
            self.twitter_accounts.c.twitter_account_id == twitter_account_id
        )
        with self.get_connection() as connection:
            connection.execute(delete_twitter_account_stmt)

    def insert_user(self, squeak_user: SqueakUser) -> int:
        """ Insert a new user. """
        ins = self.users.insert().values(
            created_time_ms=self.timestamp_now_ms,
            username=squeak_user.username,
            password_hash=squeak_user.password_hash,
        )
        with self.get_connection() as connection:
            res = connection.execute(ins)
            user_id = res.inserted_primary_key[0]
            return user_id

    def get_users(self) -> List[SqueakUser]:
        """ Get all users. """
        s = select([self.users])
        with self.get_connection() as connection:
            result = connection.execute(s)
            rows = result.fetchall()
            profiles = [self._parse_squeak_user(row) for row in rows]
            return profiles

    def get_user_by_username(self, username: str) -> Optional[SqueakUser]:
        """ Get a user by username. """
        s = select([self.users]).where(
            self.users.c.username == username,
        )
        with self.get_connection() as connection:
            result = connection.execute(s)
            row = result.fetchone()
            if row is None:
                return None
            return self._parse_squeak_user(row)

    def _parse_squeak(self, row) -> CBaseSqueak:
        if row["resqueak_hash"]:
            return CResqueak.deserialize(row["squeak"])
        else:
            return CSqueak.deserialize(row["squeak"])

    def _parse_squeak_entry(self, row) -> SqueakEntry:
        public_key_bytes = row["author_public_key"]
        secret_key_column = row["secret_key"]
        is_locked = bool(secret_key_column)
        liked_time_ms = row["liked_time_ms"]
        profile = self._try_parse_squeak_profile(
            row, profiles_table=self.author_profiles)
        return SqueakEntry(
            squeak_hash=(row["hash"]),
            serialized_squeak=(row["squeak"]),
            public_key=SqueakPublicKey.from_bytes(public_key_bytes),
            block_height=row["block_height"],
            block_hash=(row["block_hash"]),
            block_time=row["block_time_s"],
            squeak_time=row["time_s"],
            is_unlocked=is_locked,
            secret_key=(row["secret_key"]),
            liked_time_ms=liked_time_ms,
            content=row["content"],
            squeak_profile=profile,
        )

    def _parse_squeak_profile(self, row, profiles_table=None) -> SqueakProfile:
        profiles_table = profiles_table if (
            profiles_table is not None) else self.profiles

        private_key_bytes = row[profiles_table.c.private_key]
        private_key = SqueakPrivateKey.from_bytes(
            private_key_bytes) if private_key_bytes else None
        return SqueakProfile(
            profile_id=row[profiles_table.c.profile_id],
            profile_name=row[profiles_table.c.profile_name],
            private_key=private_key,
            public_key=SqueakPublicKey.from_bytes(
                row[profiles_table.c.public_key]),
            following=row[profiles_table.c.following],
            profile_image=row[profiles_table.c.profile_image],
        )

    def _parse_squeak_user(self, row, users_table=None) -> SqueakUser:
        users_table = users_table if (
            users_table is not None) else self.users

        return SqueakUser(
            user_id=row[users_table.c.user_id],
            username=row[users_table.c.username],
            password_hash=row[users_table.c.password_hash],
            user_image=row[users_table.c.user_image],
        )

    def _try_parse_squeak_profile(self, row, profiles_table=None) -> Optional[SqueakProfile]:
        profiles_table = profiles_table if (
            profiles_table is not None) else self.profiles

        if row[profiles_table.c.profile_id] is None:
            return None
        return self._parse_squeak_profile(row, profiles_table=profiles_table)

    def _parse_squeak_peer(self, row) -> SqueakPeer:
        return SqueakPeer(
            peer_id=row[self.peers.c.peer_id],
            peer_name=row["peer_name"],
            address=PeerAddress(
                network=Network[row["network"]],
                host=row["host"],
                port=row["port"],
            ),
            autoconnect=row["autoconnect"],
            share_for_free=row["share_for_free"],
        )

    def _parse_received_offer(self, row) -> ReceivedOffer:
        return ReceivedOffer(
            received_offer_id=row["received_offer_id"],
            squeak_hash=(row["squeak_hash"]),
            payment_hash=(row["payment_hash"]),
            nonce=(row["nonce"]),
            payment_point=(row["payment_point"]),
            invoice_timestamp=row["invoice_timestamp"],
            invoice_expiry=row["invoice_expiry"],
            price_msat=row["price_msat"],
            payment_request=row["payment_request"],
            destination=row["destination"],
            lightning_address=LightningAddressHostPort(
                host=row["lightning_host"],
                port=row["lightning_port"],
            ),
            peer_address=PeerAddress(
                network=Network[row["peer_network"]],
                host=row["peer_host"],
                port=row["peer_port"],
            ),
            paid=row["paid"],
        )

    def _parse_sent_payment(self, row) -> SentPayment:
        return SentPayment(
            sent_payment_id=row[self.sent_payments.c.sent_payment_id],
            created_time_ms=row[self.sent_payments.c.created_time_ms],
            peer_address=PeerAddress(
                network=Network[row["peer_network"]],
                host=row["peer_host"],
                port=row["peer_port"],
            ),
            squeak_hash=(row["squeak_hash"]),
            payment_hash=(row["payment_hash"]),
            secret_key=(row["secret_key"]),
            price_msat=row["price_msat"],
            node_pubkey=row["node_pubkey"],
            valid=row["valid"],
        )

    def _parse_sent_offer(self, row) -> SentOffer:
        return SentOffer(
            sent_offer_id=row["sent_offer_id"],
            squeak_hash=(row["squeak_hash"]),
            payment_hash=(row["payment_hash"]),
            nonce=(row["nonce"]),
            price_msat=row["price_msat"],
            payment_request=row["payment_request"],
            invoice_time=row["invoice_timestamp"],
            invoice_expiry=row["invoice_expiry"],
            peer_address=PeerAddress(
                network=Network[row["peer_network"]],
                host=row["peer_host"],
                port=row["peer_port"],
            ),
            paid=row["paid"],
        )

    def _parse_received_payment(self, row) -> ReceivedPayment:
        return ReceivedPayment(
            received_payment_id=row["received_payment_id"],
            created_time_ms=row["created_time_ms"],
            squeak_hash=(row["squeak_hash"]),
            payment_hash=(row["payment_hash"]),
            price_msat=row["price_msat"],
            settle_index=row["settle_index"],
            peer_address=PeerAddress(
                network=Network[row["peer_network"]],
                host=row["peer_host"],
                port=row["peer_port"],
            ),
        )

    def _parse_received_payment_summary(self, row) -> ReceivedPaymentSummary:
        return ReceivedPaymentSummary(
            num_received_payments=row["num_payments_received"],
            total_amount_received_msat=row["total_amount_received_msat"],
        )

    def _parse_sent_payment_summary(self, row) -> SentPaymentSummary:
        return SentPaymentSummary(
            num_sent_payments=row["num_payments_sent"],
            total_amount_sent_msat=row["total_amount_sent_msat"],
        )

    def _parse_user_config(self, row) -> UserConfig:
        return UserConfig(
            username=row["username"],
            sell_price_msat=row["sell_price_msat"],
        )

    def _parse_twitter_account_entry(self, row) -> TwitterAccountEntry:
        profile = self._try_parse_squeak_profile(row)
        return TwitterAccountEntry(
            twitter_account_id=row["twitter_account_id"],
            handle=row["handle"],
            profile_id=row["profile_id"],
            bearer_token=row["bearer_token"],
            profile=profile,
            is_forwarding=False,
        )
