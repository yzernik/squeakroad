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
from pathlib import Path

from sqlalchemy import create_engine


DB_FILE = "data-v4.db"


def get_engine(connection_string):
    return create_engine(connection_string)


def get_sqlite_connection_string(sqk_dir, network):
    data_dir = Path(sqk_dir).joinpath("data").joinpath(network)
    data_dir.mkdir(parents=True, exist_ok=True)
    return "sqlite:////{}/{}".format(
        data_dir,
        DB_FILE,
    )


def get_connection_string(config, network):
    if config.db.connection_string:
        return config.db.connection_string
    return get_sqlite_connection_string(
        config.node.sqk_dir_path,
        network,
    )
