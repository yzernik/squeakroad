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
import logging
from pathlib import Path

from typedconfig import Config
from typedconfig import group_key
from typedconfig import key
from typedconfig import section
from typedconfig.source import DictConfigSource
from typedconfig.source import EnvironmentConfigSource
from typedconfig.source import IniFileConfigSource


logger = logging.getLogger(__name__)


DEFAULT_NETWORK = "testnet"
DEFAULT_PRICE_MSAT = 10000
DEFAULT_LOG_LEVEL = "INFO"
DEFAULT_MAX_SQUEAKS = 10000
DEFAULT_MAX_SQUEAKS_PER_PUBLIC_KEY_PER_BLOCK = 100
DEFAULT_SERVER_RPC_HOST = "0.0.0.0"
DEFAULT_SERVER_RPC_PORT = 8555
DEFAULT_EXTERNAL_PORT = 8555
DEFAULT_ADMIN_RPC_HOST = "0.0.0.0"
DEFAULT_ADMIN_RPC_PORT = 8994
DEFAULT_WEBADMIN_HOST = "0.0.0.0"
DEFAULT_WEBADMIN_PORT = 12994
DEFAULT_BITCOIN_RPC_HOST = "localhost"
DEFAULT_BITCOIN_RPC_PORT = 18334
BITCOIN_RPC_PORT = {
    "mainnet": 8332,
    "testnet": 18332,
    "simnet": 18556,
}
DEFAULT_LIGHTNING_BACKEND = "lnd"
DEFAULT_LND_PORT = 9735
DEFAULT_LND_RPC_PORT = 10009
DEFAULT_CLIGHTNING_DIR = ".lightning"
DEFAULT_CLIGHTNING_RPC_FILE = str(
    Path.home() / DEFAULT_CLIGHTNING_DIR / 'lightning-rpc')
DEFAULT_SQK_DIR = ".sqk"
DEFAULT_SQK_DIR_PATH = str(Path.home() / DEFAULT_SQK_DIR)
DEFAULT_LND_RPC_HOST = "localhost"
DEFAULT_INTEREST_BLOCK_INTERVAL = 2016
DEFAULT_SENT_OFFER_RETENTION_S = 86400
DEFAULT_RECEIVED_OFFER_RETENTION_S = 86400
DEFAULT_OFFER_DELETION_INTERVAL_S = 10
DEFAULT_PEER_DOWNLOAD_INTERVAL_S = 30
DEFAULT_SUBSCRIBE_INVOICES_RETRY_S = 10
DEFAULT_SQUEAK_RETENTION_S = 604800
DEFAULT_SQUEAK_DELETION_INTERVAL_S = 10
DEFAULT_FORWARD_TWEETS_RETRY_S = 10


@section('bitcoin')
class BitcoinConfig(Config):
    rpc_host = key(cast=str, required=False, default=DEFAULT_BITCOIN_RPC_HOST)
    rpc_port = key(cast=int, required=False, default=DEFAULT_BITCOIN_RPC_PORT)
    rpc_user = key(cast=str, required=False, default="")
    rpc_pass = key(cast=str, required=False, default="")
    rpc_use_ssl = key(cast=bool, required=False, default=False)
    rpc_ssl_cert = key(cast=str, required=False, default="")


@section('lightning')
class LightningConfig(Config):
    backend = key(cast=str, required=False, default=DEFAULT_LIGHTNING_BACKEND)
    external_host = key(cast=str, required=False, default="")
    external_port = key(cast=int, required=False, default=DEFAULT_LND_PORT)
    lnd_rpc_host = key(cast=str, required=False, default=DEFAULT_LND_RPC_HOST)
    lnd_rpc_port = key(cast=int, required=False, default=DEFAULT_LND_RPC_PORT)
    lnd_tls_cert_path = key(cast=str, required=False, default="")
    lnd_macaroon_path = key(cast=str, required=False, default="")
    clightning_rpc_file = key(cast=str, required=False,
                              default=DEFAULT_CLIGHTNING_RPC_FILE)


@section('tor')
class TorConfig(Config):
    proxy_ip = key(cast=str, required=False, default="")
    proxy_port = key(cast=int, required=False, default=0)


@section('server')
class ServerConfig(Config):
    enabled = key(cast=bool, required=False, default=True)
    host = key(cast=str, required=False, default=DEFAULT_SERVER_RPC_HOST)
    port = key(cast=int, required=False, default=DEFAULT_SERVER_RPC_PORT)
    external_address = key(cast=str, required=False, default="")
    external_port = key(cast=int, required=False,
                        default=DEFAULT_EXTERNAL_PORT)


@section('rpc')
class RpcConfig(Config):
    enabled = key(cast=bool, required=False, default=False)
    host = key(cast=str, required=False, default=DEFAULT_ADMIN_RPC_HOST)
    port = key(cast=int, required=False, default=DEFAULT_ADMIN_RPC_PORT)


@section('webadmin')
class WebadminConfig(Config):
    enabled = key(cast=bool, required=False, default=False)
    host = key(cast=str, required=False, default=DEFAULT_WEBADMIN_HOST)
    port = key(cast=int, required=False, default=DEFAULT_WEBADMIN_PORT)
    username = key(cast=str, required=False, default="")
    password = key(cast=str, required=False, default="")
    use_ssl = key(cast=bool, required=False, default=False)
    login_disabled = key(cast=bool, required=False, default=False)
    allow_cors = key(cast=bool, required=False, default=False)


@section('node')
class NodeConfig(Config):
    network = key(
        cast=str, required=False, default=DEFAULT_NETWORK)
    price_msat = key(
        cast=int, required=False, default=DEFAULT_PRICE_MSAT)
    max_squeaks = key(
        cast=int, required=False, default=DEFAULT_MAX_SQUEAKS)
    max_squeaks_per_public_key_per_block = key(
        cast=int, required=False, default=DEFAULT_MAX_SQUEAKS_PER_PUBLIC_KEY_PER_BLOCK)
    sqk_dir_path = key(
        cast=str, required=False, default=DEFAULT_SQK_DIR_PATH)
    log_level = key(
        cast=str, required=False, default=DEFAULT_LOG_LEVEL)
    sent_offer_retention_s = key(
        cast=int, required=False, default=DEFAULT_SENT_OFFER_RETENTION_S)
    received_offer_retention_s = key(
        cast=int, required=False, default=DEFAULT_RECEIVED_OFFER_RETENTION_S)
    subscribe_invoices_retry_s = key(
        cast=int, required=False, default=DEFAULT_SUBSCRIBE_INVOICES_RETRY_S)
    squeak_retention_s = key(
        cast=int, required=False, default=DEFAULT_SQUEAK_RETENTION_S)
    squeak_deletion_interval_s = key(
        cast=int, required=False, default=DEFAULT_SQUEAK_DELETION_INTERVAL_S)
    offer_deletion_interval_s = key(
        cast=int, required=False, default=DEFAULT_OFFER_DELETION_INTERVAL_S)
    interest_block_interval = key(
        cast=int, required=False, default=DEFAULT_INTEREST_BLOCK_INTERVAL)
    peer_download_interval_s = key(
        cast=int, required=False, default=DEFAULT_PEER_DOWNLOAD_INTERVAL_S)


@section('db')
class DbConfig(Config):
    connection_string = key(cast=str, required=False, default="")


@section('twitter')
class TwitterConfig(Config):
    forward_tweets_retry_s = key(
        cast=int, required=False, default=DEFAULT_FORWARD_TWEETS_RETRY_S)


class SqueaknodeConfig(Config):
    bitcoin = group_key(BitcoinConfig)
    lightning = group_key(LightningConfig)
    tor = group_key(TorConfig)
    server = group_key(ServerConfig)
    rpc = group_key(RpcConfig)
    webadmin = group_key(WebadminConfig)
    node = group_key(NodeConfig)
    db = group_key(DbConfig)
    twitter = group_key(TwitterConfig)
    # description = key(cast=str, section_name="general")

    def __init__(self, config_path=None, dict_config=None):
        super().__init__()
        self.prefix = "SQUEAKNODE"
        self.config_path = config_path
        self.dict_config = dict_config

    def read(self):
        if self.dict_config is not None:
            self.add_source(DictConfigSource(self.dict_config))
        self.add_source(EnvironmentConfigSource(prefix=self.prefix))
        if self.config_path is not None:
            self.add_source(IniFileConfigSource(self.config_path))
        return super().read()
