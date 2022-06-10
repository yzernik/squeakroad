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
from typing import Optional

from squeaknode.core.user_config import UserConfig


logger = logging.getLogger(__name__)

NODE_SETTINGS_USERNAME = "default"


class NodeSettings:

    def __init__(self, squeak_db):
        self.squeak_db = squeak_db
        self.username = NODE_SETTINGS_USERNAME

    def insert_user_config(self) -> Optional[str]:
        user_config = UserConfig(username=self.username)
        return self.squeak_db.insert_config(user_config)

    def set_sell_price_msat(self, sell_price_msat: int) -> None:
        self.insert_user_config()
        if sell_price_msat < 0:
            raise Exception("Sell price cannot be negative.")
        self.squeak_db.set_config_sell_price_msat(
            username=self.username,
            sell_price_msat=sell_price_msat,
        )

    def clear_sell_price_msat(self) -> None:
        self.insert_user_config()
        self.squeak_db.clear_config_sell_price_msat(
            username=self.username,
        )

    def get_sell_price_msat(self) -> Optional[int]:
        user_config = self.squeak_db.get_config(
            username=self.username,
        )
        if user_config is None:
            return None
        return user_config.sell_price_msat
