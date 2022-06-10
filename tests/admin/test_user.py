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

from squeaknode.admin.squeak_admin_server_handler import SqueakAdminServerHandler
from squeaknode.admin.webapp.user import UserLookup


@pytest.fixture
def username():
    yield "fake_username"


@pytest.fixture
def password():
    yield "fake_password"


@pytest.fixture
def handler():
    yield SqueakAdminServerHandler(
        lightning_client=None,
        squeak_controller=None,
    )


@pytest.fixture
def user_lookup(username, password, handler):
    yield UserLookup(username, password, handler)


@pytest.fixture
def user(user_lookup, username):
    yield user_lookup.get_user_by_username(username)


def test_user_lookup(handler, user_lookup, username):
    # with mock.patch.object(handler, '', autospec=True) as mock_make_request:
    #     mock_make_request.return_value = mock_get_block_header_response.json()

    assert user_lookup.get_user_by_username(username) is not None
    assert user_lookup.get_user_by_username("wrong_username") is None


def test_user(user, username, password):

    assert user.is_authenticated()
    assert user.is_active()
    assert not user.is_anonymous()
    assert user.get_id() == username
    assert user.check_password(password)
    assert not user.check_password("wrong_password")
