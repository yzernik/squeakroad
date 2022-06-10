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

from squeaknode.core.profiles import create_contact_profile
from squeaknode.core.profiles import create_signing_profile
from squeaknode.core.profiles import get_profile_private_key


@pytest.fixture
def profile_name():
    yield "fake_profile_name"


# @pytest.fixture
# def invalid_public_key():
#     yield "abcdefg"


def test_create_signing_profile(profile_name):
    profile = create_signing_profile(profile_name)

    assert profile.profile_name == profile_name


def test_create_signing_profile_empty_name():
    with pytest.raises(Exception) as excinfo:
        create_signing_profile("")
    assert "Profile name cannot be empty." in str(excinfo.value)


def test_import_signing_profile(profile_name, private_key, public_key):
    profile = create_signing_profile(profile_name, private_key)

    assert profile.profile_name == profile_name
    assert profile.private_key == private_key
    assert profile.public_key == public_key


def test_create_contact_profile(profile_name, public_key):
    profile = create_contact_profile(profile_name, public_key)

    assert profile.profile_name == profile_name
    assert profile.public_key == public_key


# def test_create_contact_profile_invalid_public_key(profile_name, invalid_public_key):
#     with pytest.raises(Exception) as excinfo:
#         create_contact_profile(profile_name, invalid_address_str)
#     assert "Invalid squeak address" in str(excinfo.value)


def test_get_profile_private_key(signing_profile, private_key):
    private_key_from_profile = get_profile_private_key(signing_profile)

    assert private_key_from_profile == private_key


def test_get_profile_private_key_missing(contact_profile):
    with pytest.raises(Exception) as excinfo:
        get_profile_private_key(contact_profile)
    assert "does not have a private key" in str(excinfo.value)
