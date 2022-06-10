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
import logging
from typing import List

import requests

from squeaknode.core.tweet_stream import TweetStream


logger = logging.getLogger(__name__)


class TwitterStream:

    twitter_stream_url = "https://api.twitter.com/2/tweets/search/stream"
    twitter_stream_rules_url = "https://api.twitter.com/2/tweets/search/stream/rules"

    def __init__(self, bearer_token: str, handles: List[str]):
        self.bearer_token = bearer_token
        self.handles = handles

    def get_tweets(self) -> TweetStream:
        rules = self.get_rules()
        delete = self.delete_all_rules(rules)
        set = self.set_rules(delete)
        return self.get_stream(set)

    @property
    def bearer_oauth_fn(self):
        def bearer_oauth(r):
            """
            Method required by bearer token authentication.
            """

            r.headers["Authorization"] = f"Bearer {self.bearer_token}"
            r.headers["User-Agent"] = "v2FilteredStreamPython"
            return r
        return bearer_oauth

    def get_rules(self):
        response = requests.get(
            self.twitter_stream_rules_url,
            auth=self.bearer_oauth_fn
        )
        if response.status_code != 200:
            raise Exception(
                "Cannot get rules (HTTP {}): {}".format(
                    response.status_code, response.text)
            )
        logger.info(json.dumps(response.json()))
        return response.json()

    def delete_all_rules(self, rules):
        if rules is None or "data" not in rules:
            return None

        ids = list(map(lambda rule: rule["id"], rules["data"]))
        payload = {"delete": {"ids": ids}}
        response = requests.post(
            self.twitter_stream_rules_url,
            auth=self.bearer_oauth_fn,
            json=payload
        )
        if response.status_code != 200:
            raise Exception(
                "Cannot delete rules (HTTP {}): {}".format(
                    response.status_code, response.text
                )
            )
        logger.info(json.dumps(response.json()))

    def set_rules(self, delete):
        sample_rules = [
            {"value": f"from:{handle}", "tag": handle}
            for handle in self.handles
        ]
        payload = {"add": sample_rules}
        response = requests.post(
            self.twitter_stream_rules_url,
            auth=self.bearer_oauth_fn,
            json=payload,
        )
        if response.status_code != 201:
            raise Exception(
                "Cannot add rules (HTTP {}): {}".format(
                    response.status_code, response.text)
            )
        logger.info(json.dumps(response.json()))

    def get_stream(self, set) -> TweetStream:
        response = requests.get(
            self.twitter_stream_url,
            auth=self.bearer_oauth_fn,
            stream=True,
        )
        logger.info(response.status_code)
        if response.status_code != 200:
            raise Exception(
                "Cannot get stream (HTTP {}): {}".format(
                    response.status_code, response.text
                )
            )
        result_stream = (
            json.loads(response_line)
            for response_line in response.iter_lines()
            if response_line
        )
        return TweetStream(
            cancel_fn=response.close,
            result_stream=result_stream,
        )
