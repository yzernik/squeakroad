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

from squeak.params import SelectParams

from squeaknode.admin.squeak_admin_server_handler import SqueakAdminServerHandler
from squeaknode.admin.webapp.app import SqueakAdminWebServer
from squeaknode.bitcoin.bitcoin_core_client import BitcoinCoreClient
from squeaknode.config.config import SqueaknodeConfig
from squeaknode.core.squeak_core import SqueakCore
from squeaknode.db.db_engine import get_connection_string
from squeaknode.db.db_engine import get_engine
from squeaknode.db.squeak_db import SqueakDb
from squeaknode.lightning.clightning_lightning_client import CLightningClient
from squeaknode.lightning.lnd_lightning_client import LNDLightningClient
from squeaknode.node.node_settings import NodeSettings
from squeaknode.node.payment_processor import PaymentProcessor
from squeaknode.node.process_received_payments_worker import ProcessReceivedPaymentsWorker
from squeaknode.node.squeak_controller import SqueakController
from squeaknode.node.squeak_deletion_worker import SqueakDeletionWorker
from squeaknode.node.squeak_offer_expiry_worker import SqueakOfferExpiryWorker
from squeaknode.node.squeak_store import SqueakStore


logger = logging.getLogger(__name__)


class SqueakNode:

    def __init__(self, config: SqueaknodeConfig):
        self.config = config
        self.set_network_params()
        self.create_db()
        self.create_node_settings()
        self.create_lightning_client()
        self.create_bitcoin_client()
        self.create_squeak_core()
        self.create_squeak_store()
        self.create_payment_processor()
        self.create_squeak_controller()
        self.create_admin_handler()
        self.create_admin_web_server()
        self.create_received_payment_processor_worker()
        self.create_squeak_deletion_worker()
        self.create_offer_expiry_worker()

    def start_running(self):
        self.squeak_db.init_with_retries()
        self.lightning_client.init()
        if self.config.webadmin.enabled:
            self.admin_web_server.start()
        self.received_payment_processor_worker.start_running()
        self.squeak_deletion_worker.start()
        self.offer_expiry_worker.start()

    def stop_running(self):
        self.admin_web_server.stop()
        self.received_payment_processor_worker.stop_running()

    def set_network_params(self):
        SelectParams(self.config.node.network)

    def create_db(self):
        connection_string = get_connection_string(
            self.config,
            self.config.node.network,
        )
        logger.info("Using connection string: {}".format(
            connection_string))
        engine = get_engine(connection_string)
        self.squeak_db = SqueakDb(engine)

    def create_node_settings(self):
        self.node_settings = NodeSettings(self.squeak_db)

    def create_lightning_client(self):
        if self.config.lightning.backend == 'lnd':
            self.lightning_client = LNDLightningClient(
                self.config.lightning.lnd_rpc_host,
                self.config.lightning.lnd_rpc_port,
                self.config.lightning.lnd_tls_cert_path,
                self.config.lightning.lnd_macaroon_path,
            )
        elif self.config.lightning.backend == 'clightning':
            self.lightning_client = CLightningClient(
                self.config.lightning.clightning_rpc_file,
            )
        else:
            raise Exception('Invalid lightning backend: {}'.format(
                self.config.lightning.backend,
            ))

    def create_bitcoin_client(self):
        self.bitcoin_client = BitcoinCoreClient(
            self.config.bitcoin.rpc_host,
            self.config.bitcoin.rpc_port,
            self.config.bitcoin.rpc_user,
            self.config.bitcoin.rpc_pass,
            self.config.bitcoin.rpc_use_ssl,
            self.config.bitcoin.rpc_ssl_cert,
        )

    def create_squeak_core(self):
        self.squeak_core = SqueakCore(
            self.bitcoin_client,
            self.lightning_client,
        )

    def create_squeak_store(self):
        self.squeak_store = SqueakStore(
            self.squeak_db,
            self.squeak_core,
            self.config.node.max_squeaks,
            self.config.node.max_squeaks_per_public_key_per_block,
            self.config.node.squeak_retention_s,
            self.config.node.received_offer_retention_s,
            self.config.node.sent_offer_retention_s,
        )

    def create_payment_processor(self):
        self.payment_processor = PaymentProcessor(
            self.squeak_db,
            self.squeak_core,
            self.config.node.subscribe_invoices_retry_s,
        )

    def create_squeak_controller(self):
        self.squeak_controller = SqueakController(
            self.squeak_store,
            self.payment_processor,
            self.node_settings,
            self.config,
        )

    def create_admin_handler(self):
        self.admin_handler = SqueakAdminServerHandler(
            self.lightning_client,
            self.squeak_controller,
        )

    def create_admin_web_server(self):
        self.admin_web_server = SqueakAdminWebServer(
            self.config.webadmin.host,
            self.config.webadmin.port,
            self.config.webadmin.username,
            self.config.webadmin.password,
            self.config.webadmin.use_ssl,
            self.config.webadmin.login_disabled,
            self.config.webadmin.allow_cors,
            self.admin_handler,
        )

    def create_received_payment_processor_worker(self):
        self.received_payment_processor_worker = ProcessReceivedPaymentsWorker(
            self.payment_processor,
        )

    def create_squeak_deletion_worker(self):
        self.squeak_deletion_worker = SqueakDeletionWorker(
            self.squeak_store,
            self.config.node.squeak_deletion_interval_s,
        )

    def create_offer_expiry_worker(self):
        self.offer_expiry_worker = SqueakOfferExpiryWorker(
            self.squeak_store,
            self.config.node.offer_deletion_interval_s,
        )
