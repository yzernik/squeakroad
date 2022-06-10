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
from squeaknode.core.peer_address import PeerAddress
from squeaknode.core.squeak_peer import SqueakPeer


def create_saved_peer(
        peer_name: str,
        peer_address: PeerAddress,
        default_peer_port: int,
) -> SqueakPeer:
    validate_saved_peer_name(peer_name)
    if peer_address.port == 0:
        peer_address = peer_address._replace(
            port=default_peer_port,
        )
    return SqueakPeer(
        peer_id=None,
        peer_name=peer_name,
        address=peer_address,
        autoconnect=True,
        share_for_free=False,
    )


def validate_saved_peer_name(peer_name: str) -> None:
    """Check if the given name is valid for a peer.

    Raise exception if name is invalid.
    """
