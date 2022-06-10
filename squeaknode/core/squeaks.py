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
from typing import Optional
from typing import Tuple

from squeak.core import CheckSqueak
from squeak.core import CResqueak
from squeak.core import CSqueak
from squeak.core import MakeResqueak
from squeak.core import MakeSqueakFromStr
from squeak.core.elliptic import payment_point_bytes_from_scalar_bytes
from squeak.core.keys import SqueakPrivateKey
from squeak.core.keys import SqueakPublicKey


DATA_KEY_LENGTH = 32
VERSION_NONCE_LENGTH = 8

HASH_LENGTH = 32
EMPTY_HASH = b'\x00' * HASH_LENGTH


def get_hash(squeak: CSqueak):
    """Get the hash of the given squeak.

    Returns the bytes in the reverse order that `GetHash` of squeaklib
    returns, because the convention is to use the opposite endian-ness.

    Args:
        squeak: The given squeak object

    Returns:
        bytes: the squeak that was created together
    """
    return squeak.GetHash()[::-1]


def make_squeak_with_block(
        private_key: SqueakPrivateKey,
        content_str: str,
        block_height: int,
        block_hash: bytes,
        replyto_hash: Optional[bytes] = None,
        recipient_public_key: Optional[SqueakPublicKey] = None,
) -> Tuple[CSqueak, bytes]:
    """Create a new squeak.

    Args:
        private_key: The private key to sign the squeak.
        content_str: The content of the squeak as a string.
        block_height: The height of the latest block in the bitcoin blockchain.
        block_hash: The hash of the latest block in the bitcoin blockchain.
        replyto_hash: The hash of the squeak to which this one is replying.
        recipient_public_key: The public key of the recipient of a private squeak.

    Returns:
        Tuple[CSqueak, bytes]: the squeak that was created together
    with its decryption key.
    """
    timestamp = int(time.time())
    return MakeSqueakFromStr(
        private_key,
        content_str,
        block_height,
        block_hash,
        timestamp,
        reply_to=replyto_hash,
        recipient=recipient_public_key,
    )


def make_resqueak_with_block(
        private_key: SqueakPrivateKey,
        resqueak_hash: bytes,
        block_height: int,
        block_hash: bytes,
        replyto_hash: Optional[bytes] = None,
) -> CResqueak:
    """Create a new resqueak.

    Args:
        private_key: The private key to sign the squeak.
        resqueak_hash: The hash of the squeak to resqueak.
        block_height: The height of the latest block in the bitcoin blockchain.
        block_hash: The hash of the latest block in the bitcoin blockchain.
        replyto_hash: The hash of the squeak to which this one is replying.
        recipient_public_key: The public key of the recipient of a private squeak.

    Returns:
        CResqueak: the resqueak that was created.
    """
    timestamp = int(time.time())
    return MakeResqueak(
        private_key,
        resqueak_hash,
        block_height,
        block_hash,
        timestamp,
        reply_to=replyto_hash,
    )


def check_squeak(squeak: CSqueak) -> None:
    """Checks if the squeak is valid and has a valid signature.

    Args:
        squeak: The squeak to be validated.

    Returns:
        None

    Raises:
        Exception: If the squeak is not valid.
    """
    CheckSqueak(squeak)


# TODO: return bytes (encoded utf-8 content)
def get_decrypted_content(
        squeak: CSqueak,
        secret_key: bytes,
        authorPrivKey: Optional[SqueakPrivateKey] = None,
        recipientPrivKey: Optional[SqueakPrivateKey] = None,
) -> str:
    """Checks if the secret key is valid for the given squeak and returns
    the decrypted content.

    Args:
        squeak: The squeak to be validated.
        secret_key: The secret key.

    Returns:
        bytes: the decrypted content

    Raises:
        Exception: If the secret key is not valid.
    """
    return squeak.GetDecryptedContentStr(
        secret_key,
        authorPrivKey=authorPrivKey,
        recipientPrivKey=recipientPrivKey,
    )


def get_payment_point_of_secret_key(secret_key: bytes) -> bytes:
    """Get the payment point of the given secret key scalar.

    Args:
        secret_key: The secret key.

    Returns:
        bytes: the payment point in compressed format.
    """
    return payment_point_bytes_from_scalar_bytes(secret_key)
