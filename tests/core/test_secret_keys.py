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
import pytest
from squeak.core.elliptic import generate_secret_key

from squeaknode.core.secret_keys import add_tweak
from squeaknode.core.secret_keys import generate_tweak
from squeaknode.core.secret_keys import subtract_tweak


@pytest.fixture
def secret_key():
    yield generate_secret_key()


@pytest.fixture
def tweak():
    yield generate_tweak()


def test_add_subtract_tweak(secret_key, tweak):
    tweaked_secret_key = add_tweak(secret_key, tweak)
    untweaked_secret_key = subtract_tweak(tweaked_secret_key, tweak)

    assert untweaked_secret_key == secret_key
