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
from typing import Callable
from typing import Optional
from typing import Tuple

from bitcoin.core import CBlockHeader
from squeak.core import CBaseSqueak
from squeak.core import CSqueak

from squeaknode.bitcoin.bitcoin_client import BitcoinClient
from squeaknode.core.lightning_address import LightningAddressHostPort
from squeaknode.core.offer import Offer
from squeaknode.core.peer_address import PeerAddress
from squeaknode.core.received_offer import ReceivedOffer
from squeaknode.core.received_payment import ReceivedPayment
from squeaknode.core.received_payment_stream import ReceivedPaymentsStream
from squeaknode.core.secret_keys import add_tweak
from squeaknode.core.secret_keys import generate_tweak
from squeaknode.core.secret_keys import subtract_tweak
from squeaknode.core.sent_offer import SentOffer
from squeaknode.core.sent_payment import SentPayment
from squeaknode.core.squeak_profile import SqueakProfile
from squeaknode.core.squeaks import check_squeak
from squeaknode.core.squeaks import get_decrypted_content
from squeaknode.core.squeaks import get_hash
from squeaknode.core.squeaks import get_payment_point_of_secret_key
from squeaknode.core.squeaks import make_resqueak_with_block
from squeaknode.core.squeaks import make_squeak_with_block
from squeaknode.lightning.lightning_client import LightningClient


logger = logging.getLogger(__name__)


class SqueakCore:
    def __init__(
        self,
        bitcoin_client: BitcoinClient,
        lightning_client: LightningClient,
    ):
        self.bitcoin_client = bitcoin_client
        self.lightning_client = lightning_client

    def make_squeak(
            self,
            signing_profile: SqueakProfile,
            content_str: str,
            replyto_hash: Optional[bytes] = None,
            recipient_profile: Optional[SqueakProfile] = None,
    ) -> Tuple[CSqueak, bytes]:
        """Create a new squeak.

        Args:
            signing_profile: The profile of the author of the squeak.
            content_str: The content of the squeak as a string.
            replyto_hash: The hash of the squeak to which this one is replying.

        Returns:
            Tuple[CSqueak, bytes]: the squeak that was created together
        with its decryption key.

        Raises:
            Exception: If the profile does not have a signing key.
        """
        if signing_profile.private_key is None:
            raise Exception("Can't make squeak with a contact profile.")
        block_info = self.bitcoin_client.get_best_block_info()
        block_height = block_info.block_height
        block_hash = block_info.block_hash
        squeak, secret_key = make_squeak_with_block(
            signing_profile.private_key,
            content_str,
            block_height,
            block_hash,
            replyto_hash=replyto_hash,
            recipient_public_key=recipient_profile.public_key if recipient_profile else None,
        )
        return squeak, secret_key

    def make_resqueak(
            self,
            signing_profile: SqueakProfile,
            resqueak_hash: bytes,
            replyto_hash: Optional[bytes] = None,
    ) -> Tuple[CSqueak, bytes]:
        """Create a new resqueak.

        Args:
            signing_profile: The profile of the author of the squeak.
            resqueak_hash: The hash of the squeak to resqueak.
            replyto_hash: The hash of the squeak to which this one is replying.

        Returns:
            CResqueak: the resqueak that was created.

        Raises:
            Exception: If the profile does not have a signing key.
        """
        if signing_profile.private_key is None:
            raise Exception("Can't make squeak with a contact profile.")
        block_info = self.bitcoin_client.get_best_block_info()
        block_height = block_info.block_height
        block_hash = block_info.block_hash
        squeak = make_resqueak_with_block(
            signing_profile.private_key,
            resqueak_hash,
            block_height,
            block_hash,
            replyto_hash=replyto_hash,
        )
        return squeak

    def check_squeak(self, base_squeak: CBaseSqueak) -> None:
        """Checks if the squeak is valid and has a valid signature.

        Args:
            squeak: The squeak to be validated.

        Returns:
            None

        Raises:
            Exception: If the squeak is not valid.
        """
        check_squeak(base_squeak)

    def get_block_header(self, base_squeak: CBaseSqueak) -> CBlockHeader:
        """Checks if the embedded block hash in the squeak is valid for its
        block height and return the associtated block header.

        Args:
            squeak: The squeak to be validated.

        Returns:
            CBlockHeader: the block header associated with the given squeak.

        Raises:
            Exception: If the block hash is not valid.
        """
        block_info = self.bitcoin_client.get_block_info_by_height(
            base_squeak.nBlockHeight)
        if base_squeak.hashBlock != block_info.block_hash:
            raise Exception("Block hash incorrect.")
        return block_info.block_header

    def get_decrypted_content(
            self,
            squeak: CSqueak,
            secret_key: bytes,
            author_profile: Optional[SqueakProfile] = None,
            recipient_profile: Optional[SqueakProfile] = None,
    ) -> str:
        """Checks if the secret key is valid for the given squeak and returns
        the decrypted content.

        Args:
            squeak: The squeak to be validated.
            secret_key: The secret key.
            author_profile: The profile of the author.
            recipient_profile: The profile of the recipient.

        Returns:
            str: the decrypted content as a string.

        Raises:
            Exception: If the secret key is not valid.
        """
        if author_profile and author_profile.private_key is None:
            raise Exception("Author profile does not have private key.")
        if recipient_profile and recipient_profile.private_key is None:
            raise Exception("Recipient profile does not have private key.")
        return get_decrypted_content(
            squeak,
            secret_key,
            authorPrivKey=author_profile.private_key if author_profile else None,
            recipientPrivKey=recipient_profile.private_key if recipient_profile else None,
        )

    def get_best_block_height(self) -> int:
        """Get the current height of the latest block in the blockchain.

        Returns:
            int: the current latest block height.
        """
        block_info = self.bitcoin_client.get_best_block_info()
        return block_info.block_height

    def create_offer(
            self,
            squeak: CSqueak,
            secret_key: bytes,
            peer_address: PeerAddress,
            price_msat: int,
            nonce: bytes = None,
    ) -> SentOffer:
        """Creates an offer to sell a squeak key to another node.

        Args:
            squeak: The squeak to be sold.
            secret_key: The secret key to the squeak.
            peer_address: The address of the buyer.
            price_msat: The price in msats.

        Returns:
            SentOffer: A record of the details of the offer for the seller.
        """
        # Get the squeak hash
        squeak_hash = get_hash(squeak)
        # Generate a new random nonce
        if nonce is None:
            nonce = generate_tweak()
        # Calculate the preimage
        preimage = add_tweak(secret_key, nonce)
        # Create the lightning invoice
        invoice = self.lightning_client.create_invoice(
            preimage, price_msat
        )
        return SentOffer(
            sent_offer_id=None,
            squeak_hash=squeak_hash,
            payment_hash=invoice.r_hash,
            nonce=nonce,
            price_msat=price_msat,
            payment_request=invoice.payment_request,
            invoice_time=invoice.creation_date,
            invoice_expiry=invoice.expiry,
            peer_address=peer_address,
            paid=False,
        )

    def get_lnd_external_address(self) -> Optional[LightningAddressHostPort]:
        """Get the external address of the local lightning node.

        Returns:
            Optional[LightningAddressHostPort]: The host and port of the lnd node if possible.
        """
        info = self.lightning_client.get_info()
        for uri in info.uris:
            pubkey, address = uri.split("@")
            host, port_str = address.split(":")
            port = int(port_str)
            return LightningAddressHostPort(
                host=host,
                port=port,
            )
        return None

    def package_offer(self, sent_offer: SentOffer, lnd_external_address: Optional[LightningAddressHostPort]) -> Offer:
        """Package the offer details into a message that will be sent from
        seller to buyer.

        Args:
            sent_offer: The offer that was already generated by the seller.
            lnd_external_address: The host and port of the lnd node if available.

        Returns:
            SentOffer: A record of the details of the offer for the seller.
        """
        if lnd_external_address is None:
            lnd_external_address = self.get_lnd_external_address()
        return Offer(
            squeak_hash=sent_offer.squeak_hash,
            nonce=sent_offer.nonce,
            payment_request=sent_offer.payment_request,
            host=lnd_external_address.host if lnd_external_address else "",
            port=lnd_external_address.port if lnd_external_address else 0,
        )

    def unpack_offer(
            self,
            squeak: CSqueak,
            offer: Offer,
            peer_address: PeerAddress,
            check_payment_point: bool = False,
    ) -> ReceivedOffer:
        """Get the offer details from the message that the buyer
        receives from the seller.

        Args:
            squeak: The squeak that will be unlocked upon payment.
            offer: The offer details received from the seller.
            peer: The peer that sent the offer.

        Returns:
            ReceivedOffer: A record of the details of the offer for the buyer.
        """
        # Get the squeak hash
        squeak_hash = get_hash(squeak)
        # Check if squeak hash matches squeak_hash in buy_offer.
        if squeak_hash != offer.squeak_hash:
            raise Exception("Squeak hash in offer {!r} does not match squeak hash {!r}.".format(
                offer.squeak_hash, squeak_hash
            ))
        # Decode the payment request
        pay_req = self.lightning_client.decode_pay_req(
            offer.payment_request)
        squeak_payment_point = squeak.paymentPoint
        payment_hash = pay_req.payment_hash
        price_msat = pay_req.num_msat
        destination = pay_req.destination
        invoice_timestamp = pay_req.timestamp
        invoice_expiry = pay_req.expiry
        lightning_address = LightningAddressHostPort(
            host=offer.host or peer_address.host,
            port=offer.port,
        )
        payment_point = pay_req.payment_point
        expected_payment_point = squeak.paymentPoint
        if check_payment_point:
            if payment_point != expected_payment_point:
                raise Exception("Invalid payment point.")
        return ReceivedOffer(
            received_offer_id=None,
            squeak_hash=squeak_hash,
            price_msat=price_msat,
            payment_hash=payment_hash,
            nonce=offer.nonce,
            payment_point=squeak_payment_point,
            invoice_timestamp=invoice_timestamp,
            invoice_expiry=invoice_expiry,
            payment_request=offer.payment_request,
            destination=destination,
            lightning_address=lightning_address,
            peer_address=peer_address,
            paid=False,
        )

    def pay_offer(self, received_offer: ReceivedOffer) -> SentPayment:
        """Pay the offer that the buyer received from the seller.

        Args:
            received_offer: The details of the offer received by the buyer.

        Returns:
            SentPayment: A record of the sent payment.
        """
        # Pay the invoice
        payment = self.lightning_client.pay_invoice(
            received_offer.payment_request)
        preimage = payment.payment_preimage
        if not preimage:
            raise Exception(
                "Payment failed with error: {}".format(payment.payment_error)
            )
        # Calculate the secret key
        nonce = received_offer.nonce
        secret_key = subtract_tweak(preimage, nonce)
        # Check if the secret key is valid for the preimage
        point = get_payment_point_of_secret_key(secret_key)
        valid = point == received_offer.payment_point
        # Save the preimage of the sent payment
        # peer_address = PeerAddress(
        #     network=received_offer.peer_address.network,
        #     host=received_offer.peer_address.host,
        #     port=received_offer.peer_address.port,
        # )
        return SentPayment(
            sent_payment_id=None,
            created_time_ms=None,
            peer_address=received_offer.peer_address,
            squeak_hash=received_offer.squeak_hash,
            payment_hash=received_offer.payment_hash,
            secret_key=secret_key,
            price_msat=received_offer.price_msat,
            node_pubkey=received_offer.destination,
            valid=valid,
        )

    def get_received_payments(
            self,
            latest_settle_index: int,
            get_sent_offer_fn: Callable[[bytes], SentOffer],
    ) -> ReceivedPaymentsStream:
        """Get an iterator of received payments.

        Args:
            latest_settle_index: The latest settle index of the lnd invoice database.
            get_sent_offer_fn: Function that takes a payment hash and returns
                the corresponding SentOffer.

        Returns:
            ReceivedPaymentsStream: An object containing an iterator of received
            payments and a callback function to cancel the iteration.
        """
        # Get the stream of settled invoices.
        invoice_stream = self.lightning_client.subscribe_invoices(
            settle_index=latest_settle_index,
        )

        def cancel_subscription():
            invoice_stream.cancel()

        def get_payment_stream():
            # Yield the received payments.
            for invoice in invoice_stream.result_stream:
                payment_hash = invoice.r_hash
                settle_index = invoice.settle_index
                sent_offer = get_sent_offer_fn(payment_hash)
                if sent_offer is not None:
                    yield ReceivedPayment(
                        received_payment_id=None,
                        created_time_ms=None,
                        squeak_hash=sent_offer.squeak_hash,
                        payment_hash=sent_offer.payment_hash,
                        price_msat=sent_offer.price_msat,
                        settle_index=settle_index,
                        peer_address=sent_offer.peer_address,
                    )

        return ReceivedPaymentsStream(
            cancel_fn=cancel_subscription,
            result_stream=get_payment_stream(),
        )
