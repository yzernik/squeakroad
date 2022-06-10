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
from abc import ABC
from abc import abstractmethod

from squeaknode.bitcoin.block_info import BlockInfo

logger = logging.getLogger(__name__)


class BitcoinClient(ABC):

    @abstractmethod
    def get_best_block_info(self) -> BlockInfo:
        """Get block info for the latest Bitcoin block.

        Returns:
            BlockInfo: an object containing height, hash, and header.

        Raises:
            BitcoinRequestError: If the request fails.
        """

    @abstractmethod
    def get_block_info_by_height(self, block_height: int) -> BlockInfo:
        """Get block info for the Bitcoin block at the given height.

        Args:
            block_height: The height of the block.

        Returns:
            BlockInfo: an object containing height, hash, and header.

        Raises:
            BitcoinRequestError: If the request fails.
        """
