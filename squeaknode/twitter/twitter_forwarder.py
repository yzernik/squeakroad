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
import threading
from typing import Dict

from squeaknode.core.twitter_account_entry import TwitterAccountEntry
from squeaknode.node.squeak_store import SqueakStore
from squeaknode.twitter.twitter_stream import TwitterStream


logger = logging.getLogger(__name__)


class TwitterForwarder:

    def __init__(
            self,
            squeak_store: SqueakStore,
            retry_s: int,
    ):
        self.squeak_store = squeak_store
        self.retry_s = retry_s
        self.lock = threading.Lock()
        self.current_tasks: Dict[str, TwitterForwarderTask] = {}

    def restart(self):
        threading.Thread(
            target=self.start_processing,
            daemon=True,
        ).start()

    def start_processing(self):
        with self.lock:
            # Stop existing running tasks.
            for handle, task in list(self.current_tasks.items()):
                task.stop_processing()
                del self.current_tasks[handle]

            # Start new tasks.
            for account in self.squeak_store.get_twitter_accounts():
                task = TwitterForwarderTask(
                    self.squeak_store,
                    account,
                    self.retry_s,
                )
                task.start_processing()
                self.current_tasks[account.handle] = task

    def stop_processing(self):
        with self.lock:
            for handle, task in list(self.current_tasks.items()):
                task.stop_processing()
                del self.current_tasks[handle]

    def is_processing(self, twitter_handle: str) -> bool:
        task = self.current_tasks.get(twitter_handle)
        if not task:
            return False
        return task.is_processing()


class TwitterForwarderTask:

    def __init__(
        self,
        squeak_store: SqueakStore,
        twitter_account: TwitterAccountEntry,
        retry_s: int,
    ):
        self.squeak_store = squeak_store
        self.twitter_account = twitter_account
        self.retry_s = retry_s
        self.stopped = threading.Event()
        self.tweet_stream = None
        self.lock = threading.Lock()

    def start_processing(self):
        logger.info("Starting twitter forwarder task.")
        threading.Thread(
            target=self.process_forward_tweets,
            daemon=True,
        ).start()

    def setup_stream(self):
        logger.info("Starting Twitter stream with bearer token: {} and twitter handle: {}".format(
            self.twitter_account.bearer_token,
            self.twitter_account.handle,
        ))
        with self.lock:
            if self.stopped.is_set():
                return
            twitter_stream = TwitterStream(
                self.twitter_account.bearer_token,
                [self.twitter_account.handle],
            )
            self.tweet_stream = twitter_stream.get_tweets()

    def stop_processing(self):
        logger.info("Stopping twitter forwarder task.")
        with self.lock:
            self.stopped.set()
            if self.tweet_stream is not None:
                self.tweet_stream.cancel_fn()

    def is_processing(self) -> bool:
        return self.tweet_stream is not None

    def process_forward_tweets(self):
        # Do exponential backoff
        wait_s = self.retry_s

        while not self.stopped.is_set():
            try:
                self.setup_stream()
                for tweet in self.tweet_stream.result_stream:
                    self.handle_tweet(tweet)
            # TODO: use more specific error.
            except Exception:
                self.tweet_stream = None
                logger.exception(
                    "Unable to subscribe tweet stream. Retrying in {} seconds...".format(
                        wait_s,
                    ),
                )
                self.stopped.wait(wait_s)
                wait_s *= 2

    def is_tweet_a_match(self, tweet: dict) -> bool:
        for rule in tweet['matching_rules']:
            if rule['tag'] == self.twitter_account.handle:
                return True
        return False

    def forward_tweet(self, tweet: dict) -> None:
        self.make_squeak(
            profile_id=self.twitter_account.profile_id,
            content_str=tweet['data']['text'],
        )

    def handle_tweet(self, tweet: dict):
        logger.info(
            "Got tweet: {}".format(tweet))
        if self.is_tweet_a_match(tweet):
            self.forward_tweet(tweet)

    def make_squeak(self, profile_id: int, content_str: str):
        self.squeak_store.make_squeak(
            profile_id,
            content_str,
            None,
            None,
        )
