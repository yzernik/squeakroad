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
import mock
import pytest

from squeaknode.client.network_controller import NetworkController
from squeaknode.config.config import SqueaknodeConfig
from squeaknode.core.lightning_address import LightningAddressHostPort
from squeaknode.core.peer_address import Network
from squeaknode.core.peer_address import PeerAddress
from squeaknode.node.node_settings import NodeSettings
from squeaknode.node.payment_processor import PaymentProcessor
from squeaknode.node.squeak_controller import SqueakController
from squeaknode.node.squeak_store import SqueakStore
from squeaknode.twitter.twitter_forwarder import TwitterForwarder


@pytest.fixture
def config():
    squeaknode_config = SqueaknodeConfig()
    squeaknode_config.read()
    return squeaknode_config


@pytest.fixture
def regtest_config():
    squeaknode_config = SqueaknodeConfig(
        dict_config={'node': {'network': 'regtest'}}
    )
    squeaknode_config.read()
    return squeaknode_config


@pytest.fixture
def squeak_store():
    return mock.Mock(spec=SqueakStore)


@pytest.fixture
def node_settings():
    return mock.Mock(spec=NodeSettings)


@pytest.fixture
def lightning_host_port():
    return LightningAddressHostPort(host="my_lightning_host", port=8765)


@pytest.fixture
def peer_address():
    return PeerAddress(
        network=Network.IPV4,
        host="fake_host",
        port=5678,
    )


@pytest.fixture
def peer_address_with_zero():
    return PeerAddress(
        network=Network.IPV4,
        host="fake_host",
        port=0,
    )


@pytest.fixture
def price_msat():
    return 777


@pytest.fixture
def payment_processor():
    return mock.Mock(spec=PaymentProcessor)


@pytest.fixture
def twitter_forwarder():
    return mock.Mock(spec=TwitterForwarder)


@pytest.fixture
def network_controller():
    return mock.Mock(spec=NetworkController)


@pytest.fixture
def squeak_controller(
    squeak_store,
    payment_processor,
    twitter_forwarder,
    network_controller,
    node_settings,
    config,
):
    return SqueakController(
        squeak_store,
        payment_processor,
        twitter_forwarder,
        network_controller,
        node_settings,
        config,
    )


@pytest.fixture
def regtest_squeak_controller(
    squeak_store,
    payment_processor,
    twitter_forwarder,
    network_controller,
    node_settings,
    regtest_config,
):
    return SqueakController(
        squeak_store,
        payment_processor,
        twitter_forwarder,
        network_controller,
        node_settings,
        regtest_config,
    )


def test_get_network_default(squeak_controller):
    assert squeak_controller.get_network() == "testnet"


def test_get_network_regtest(regtest_squeak_controller):
    assert regtest_squeak_controller.get_network() == "regtest"


# def test_get_network_regtest(config, squeak_controller):
#     # with mock.patch.object(Config, 'squeaknode_network', new_callable=mock.PropertyMock) as mock_config:
#     # mock_config.return_value = 'regtest'
#     config.squeaknode_network = "regtest"
#     print(config.squeaknode_network)

#     assert squeak_controller.get_network() == "regtest"


def test_create_peer(squeak_store, squeak_controller, peer_address):
    squeak_controller.create_peer(
        "fake_peer_name",
        peer_address,
    )

    squeak_store.create_peer.assert_called_with(
        "fake_peer_name",
        peer_address,
        8555,
    )
