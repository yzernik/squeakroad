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
from abc import ABC
from abc import abstractmethod

logger = logging.getLogger(__name__)


class Worker(ABC):

    @abstractmethod
    def start(self) -> None:
        pass


class PeriodicWorker(Worker):
    """Access a bitcoin daemon using RPC."""

    @abstractmethod
    def work_fn(self) -> None:
        pass

    @abstractmethod
    def get_interval_s(self) -> int:
        pass

    @abstractmethod
    def get_name(self) -> str:
        pass

    def do_work(self):
        if self.get_interval_s():
            timer = threading.Timer(
                self.get_interval_s(),
                self.do_work,
            )
            timer.daemon = True
            timer.name = "{}_thread".format(self.get_name())
            timer.start()
            self.work_fn()

    def start(self) -> None:
        thread = threading.Thread(
            target=self.do_work,
            args=(),
        )
        thread.start()
