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
from typing import NamedTuple
from typing import Optional

from squeak.core.keys import SqueakPublicKey

from squeaknode.core.squeak_profile import SqueakProfile


class SqueakEntry(NamedTuple):
    squeak_hash: bytes
    serialized_squeak: bytes
    public_key: SqueakPublicKey
    recipient_public_key: Optional[SqueakPublicKey]
    block_height: int
    block_hash: bytes
    block_time: int
    squeak_time: int
    reply_to: Optional[bytes]
    is_unlocked: bool
    secret_key: Optional[bytes]
    squeak_profile: Optional[SqueakProfile]
    recipient_squeak_profile: Optional[SqueakProfile]
    num_replies: int
    num_resqueaks: int
    liked_time_ms: Optional[int] = None
    content: Optional[str] = None
    resqueaked_hash: Optional[bytes] = None
    resqueaked_squeak: Optional['SqueakEntry'] = None  # type: ignore
