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
import argparse
import logging
from signal import SIGHUP
from signal import SIGINT
from signal import signal
from signal import SIGTERM
from threading import Event

from squeaknode.config.config import SqueaknodeConfig
from squeaknode.node.squeak_node import SqueakNode


logger = logging.getLogger(__name__)


stop_event = Event()


def handler(signum, frame):
    stop_event.set()


def parse_args():
    parser = argparse.ArgumentParser(
        description="squeaknode runs a node using squeak protocol. ",
    )
    parser.add_argument(
        "--config",
        dest="config",
        type=str,
        help="Path to the config file.",
    )
    parser.add_argument(
        "--log-level",
        dest="log_level",
        type=str,
        default="info",
        help="Logging level",
    )
    parser.set_defaults(func=run_node)
    return parser.parse_args()


def main():
    logging.basicConfig(level=logging.ERROR)
    args = parse_args()

    # Set the log level
    level = args.log_level.upper()
    logging.getLogger().setLevel(level)

    config = SqueaknodeConfig(args.config)
    config.read()

    # Set the log level again
    level = config.node.log_level
    logging.getLogger().setLevel(level)

    args.func(config)


def run_node(config):
    logger.info("Config: {}".format(config))
    signal(SIGTERM, handler)
    signal(SIGINT, handler)
    signal(SIGHUP, handler)
    squeak_node = SqueakNode(config)
    logger.info("Starting squeaknode...")
    squeak_node.start_running()
    stop_event.wait()
    logger.info('Shutting down Squeaknode...')
    squeak_node.stop_running()


if __name__ == "__main__":
    main()
