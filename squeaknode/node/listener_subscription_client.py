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
import queue
import threading
import uuid
from contextlib import contextmanager
logger = logging.getLogger(__name__)
DEFAULT_MAX_QUEUE_SIZE = 1000
DEFAULT_UPDATE_INTERVAL_S = 1


class EventListener:
    def __init__(self):
        self.callbacks = {}

    def handle_new_item(self, item):
        for callback in self.callbacks.values():
            callback(item)

    def add_callback(self, name, callback):
        self.callbacks[name] = callback

    def remove_callback(self, name):
        del self.callbacks[name]

    @contextmanager
    def get_subscription(self, stopped: threading.Event):
        client = ListenerSubscriptionClient(
            stopped,
        )
        threading.Thread(
            target=client.wait_for_stopped,
        ).start()

        # Register the callback to populate the queue
        callback_name = "new_item_callback_{}".format(uuid.uuid1()),

        logger.debug("Adding callback.")
        self.add_callback(callback_name, client.enqueue_item)
        try:
            logger.debug("Yielding client.")
            yield client
        except Exception:
            logger.exception("Subscription client failed.")
        finally:
            logger.debug("Removing callback.")
            self.remove_callback(callback_name)

    def yield_items(self, stopped: threading.Event):
        with self.get_subscription(stopped) as client:
            for result in client.get_item():
                yield result


class ListenerSubscriptionClient():
    def __init__(
        self,
        stopped: threading.Event,
        max_queue_size=DEFAULT_MAX_QUEUE_SIZE,
    ):
        self.stopped = stopped
        self.q: queue.Queue = queue.Queue(max_queue_size)

    def enqueue_item(self, item):
        self.q.put(item)

    def wait_for_stopped(self):
        self.stopped.wait()
        self.q.put(None)

    def get_item(self):
        while True:
            item = self.q.get()
            if item is None:
                logger.debug("Poison pill swallowed.")
                return
            yield item
            self.q.task_done()
            logger.debug(
                "Removed item from queue. Size: {}".format(
                    self.q.qsize())
            )
