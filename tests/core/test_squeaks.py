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
from squeaknode.core.squeaks import check_squeak
from squeaknode.core.squeaks import get_decrypted_content
from squeaknode.core.squeaks import get_hash
from squeaknode.core.squeaks import get_payment_point_of_secret_key
from squeaknode.core.squeaks import make_squeak_with_block


def test_get_hash(squeak, squeak_hash):
    expected_hash = squeak.GetHash()[::-1]

    assert get_hash(squeak) == expected_hash


def test_make_squeak(squeak, block_count):
    assert squeak.nBlockHeight == block_count


def test_check_squeak(squeak):
    check_squeak(squeak)


def test_get_decrypted_content(squeak, secret_key, squeak_content):
    assert get_decrypted_content(squeak, secret_key) == squeak_content


def test_get_payment_point_of_secret_key(squeak, secret_key):
    assert get_payment_point_of_secret_key(
        secret_key) == squeak.paymentPoint


def test_make_private_squeak(private_key, squeak_content, block_info):
    squeak, _ = make_squeak_with_block(
        private_key,
        squeak_content,
        block_info.block_height,
        block_info.block_hash,
    )

    assert squeak.GetRecipientPubKey() is None
