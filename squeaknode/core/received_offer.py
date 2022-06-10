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

from squeaknode.core.lightning_address import LightningAddressHostPort
from squeaknode.core.peer_address import PeerAddress


class ReceivedOffer(NamedTuple):
    """Represents an offer received by a buyer."""
    received_offer_id: Optional[int]
    squeak_hash: bytes
    price_msat: int
    payment_hash: bytes
    nonce: bytes
    payment_point: bytes
    invoice_timestamp: int
    invoice_expiry: int
    payment_request: str
    destination: str
    lightning_address: LightningAddressHostPort
    peer_address: PeerAddress
    paid: bool
