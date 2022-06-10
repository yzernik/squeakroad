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
import json

import mock
import pytest
from requests import HTTPError
from requests.exceptions import ConnectionError
from requests.exceptions import RequestException
from requests.exceptions import Timeout

from squeaknode.bitcoin.bitcoin_core_client import BitcoinCoreClient
from squeaknode.bitcoin.block_info import BlockInfo
from squeaknode.bitcoin.exception import BitcoinRequestError


@pytest.fixture
def bitcoin_host():
    yield "fake_bitcoin_host"


@pytest.fixture
def bitcoin_port():
    yield 5678


@pytest.fixture
def bitcoin_user():
    yield "fake_user"


@pytest.fixture
def bitcoin_pass():
    yield "fake_pass"


@pytest.fixture
def bitcoin_core_client(bitcoin_host, bitcoin_port, bitcoin_user, bitcoin_pass):
    yield BitcoinCoreClient(
        host=bitcoin_host,
        port=bitcoin_port,
        rpc_user=bitcoin_user,
        rpc_password=bitcoin_pass,
        use_ssl=False,
        ssl_cert="fake_ssl_cert",
    )


class MockResponse:

    def json(self):
        return {}

    @property
    def status_code(self):
        return 200

    def raise_for_status(self):
        pass


class MockGetCountResponse(MockResponse):

    def __init__(self, block_count):
        self.block_count = block_count

    def json(self):
        return {'result': '{}'.format(self.block_count)}


class MockGetBlockHashResponse(MockResponse):

    def __init__(self, block_hash_str):
        self.block_hash_str = block_hash_str

    def json(self):
        return {'result': '{}'.format(self.block_hash_str)}


class MockGetBlockHeaderResponse(MockResponse):

    def __init__(self, block_header_str):
        self.block_header_str = block_header_str

    def json(self):
        return {'result': '{}'.format(self.block_header_str)}


class MockEmptyResponse(MockResponse):
    def json(self):
        return {}


class MockInvalidStatusResponse(MockResponse):

    def raise_for_status(self):
        raise HTTPError("Some http error", response=self)


@pytest.fixture
def mock_get_count_response(block_count):
    yield MockGetCountResponse(block_count)


@pytest.fixture
def mock_get_block_hash_response(block_hash_str):
    yield MockGetBlockHashResponse(block_hash_str)


@pytest.fixture
def mock_get_block_header_response(block_header_str):
    yield MockGetBlockHeaderResponse(block_header_str)


@pytest.fixture
def mock_empty_response():
    yield MockEmptyResponse()


@pytest.fixture
def mock_invalid_status_response():
    yield MockInvalidStatusResponse()


def test_make_request(bitcoin_host, bitcoin_port, bitcoin_user, bitcoin_pass, bitcoin_core_client, mock_empty_response):
    with mock.patch('squeaknode.bitcoin.bitcoin_core_client.requests.post', autospec=True) as mock_post:
        mock_post.return_value = mock_empty_response
        retrieved_json_response = bitcoin_core_client.make_request({})
        (bitcoin_address,) = mock_post.call_args.args
        request_json = mock_post.call_args.kwargs['data']

        assert bitcoin_host in bitcoin_address
        assert str(bitcoin_port) in bitcoin_address
        assert bitcoin_user in bitcoin_address
        assert bitcoin_pass in bitcoin_address
        assert json.loads(request_json) == {}
        assert retrieved_json_response == {}


def test_make_request_invalid_status(bitcoin_core_client, mock_invalid_status_response):
    with mock.patch('squeaknode.bitcoin.bitcoin_core_client.requests.post', autospec=True) as mock_post:
        mock_post.return_value = mock_invalid_status_response

        with pytest.raises(BitcoinRequestError):
            bitcoin_core_client.make_request({})


def test_make_request_connection_error(bitcoin_core_client, mock_invalid_status_response):
    with mock.patch('squeaknode.bitcoin.bitcoin_core_client.requests.post', autospec=True) as mock_post:
        mock_post.side_effect = ConnectionError()

        with pytest.raises(BitcoinRequestError):
            bitcoin_core_client.make_request({})


def test_make_request_timeout_error(bitcoin_core_client, mock_invalid_status_response):
    with mock.patch('squeaknode.bitcoin.bitcoin_core_client.requests.post', autospec=True) as mock_post:
        mock_post.side_effect = Timeout()

        with pytest.raises(BitcoinRequestError):
            bitcoin_core_client.make_request({})


def test_make_request_request_exception(bitcoin_core_client, mock_invalid_status_response):
    with mock.patch('squeaknode.bitcoin.bitcoin_core_client.requests.post', autospec=True) as mock_post:
        mock_post.side_effect = RequestException()

        with pytest.raises(BitcoinRequestError):
            bitcoin_core_client.make_request({})


def test_get_block_count(bitcoin_host, bitcoin_port, bitcoin_core_client, mock_get_count_response, block_count):
    with mock.patch.object(bitcoin_core_client, 'make_request', autospec=True) as mock_make_request:
        mock_make_request.return_value = mock_get_count_response.json()
        retrieved_block_count = bitcoin_core_client.get_block_count()
        (payload,) = mock_make_request.call_args.args

        assert payload['method'] == 'getblockcount'
        assert retrieved_block_count == block_count


def test_get_block_hash(bitcoin_core_client, mock_get_block_hash_response, block_count, block_hash):
    with mock.patch.object(bitcoin_core_client, 'make_request', autospec=True) as mock_make_request:
        mock_make_request.return_value = mock_get_block_hash_response.json()
        retrieved_block_hash = bitcoin_core_client.get_block_hash(block_count)
        (payload,) = mock_make_request.call_args.args

        assert payload['method'] == 'getblockhash'
        assert payload['params'] == [block_count]
        assert retrieved_block_hash == block_hash


def test_get_block_header(bitcoin_core_client, mock_get_block_header_response, block_hash, block_header):
    with mock.patch.object(bitcoin_core_client, 'make_request', autospec=True) as mock_make_request:
        mock_make_request.return_value = mock_get_block_header_response.json()
        retrieved_block_header = bitcoin_core_client.get_block_header(
            block_hash)
        (payload,) = mock_make_request.call_args.args

        assert payload['method'] == 'getblockheader'
        assert payload['params'] == [block_hash.hex(), False]
        assert retrieved_block_header == block_header


def test_get_block_info_by_height(bitcoin_core_client, block_count, block_hash, block_header):
    with mock.patch.object(bitcoin_core_client, 'get_block_hash', autospec=True) as mock_get_block_hash, \
            mock.patch.object(bitcoin_core_client, 'get_block_header', autospec=True) as mock_get_block_header:
        mock_get_block_hash.return_value = block_hash
        mock_get_block_header.return_value = block_header
        retrieved_block_info = bitcoin_core_client.get_block_info_by_height(
            block_count)

        assert mock_get_block_hash.call_args == mock.call(
            block_count)
        assert mock_get_block_header.call_args == mock.call(
            block_hash)
        assert retrieved_block_info == BlockInfo(
            block_height=block_count,
            block_hash=block_hash,
            block_header=block_header,
        )


def test_get_best_block_info(bitcoin_core_client, block_count, block_info):
    with mock.patch.object(bitcoin_core_client, 'get_block_count', autospec=True) as mock_get_block_count, \
            mock.patch.object(bitcoin_core_client, 'get_block_info_by_height', autospec=True) as mock_get_block_info_by_height:
        mock_get_block_count.return_value = block_count
        mock_get_block_info_by_height.return_value = block_info
        retrieved_block_info = bitcoin_core_client.get_best_block_info()

        assert mock_get_block_count.call_args == ()
        assert mock_get_block_info_by_height.call_args == mock.call(
            block_count)
        assert retrieved_block_info == block_info
