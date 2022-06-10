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
import os
import sys
from typing import List
from typing import Tuple

import pkg_resources
import setuptools.command.build_py
import setuptools.command.test
from setuptools import Command
from setuptools import find_packages
from setuptools import setup

from squeaknode import __version__


with open("requirements.txt") as f:
    requirements = [r for r in f.read().split("\n") if len(r)]


class BuildPyCommand(setuptools.command.build_py.build_py):
    """Custom build command."""

    def run(self):
        self.run_command('build_proto_modules')
        setuptools.command.build_py.build_py.run(self)


def build_package_protos(package_root, strict_mode=False):
    # Import grpc_tools here because it is not available at top of module.
    from grpc_tools import protoc

    proto_files = []
    inclusion_root = os.path.abspath(package_root)
    for root, _, files in os.walk(inclusion_root):
        for filename in files:
            if filename.endswith('.proto'):
                proto_files.append(os.path.abspath(os.path.join(root,
                                                                filename)))

    well_known_protos_include = pkg_resources.resource_filename(
        'grpc_tools', '_proto')

    for proto_file in proto_files:
        command = [
            'grpc_tools.protoc',
            '--proto_path={}'.format(inclusion_root),
            '--proto_path={}'.format(well_known_protos_include),
            '--python_out={}'.format(inclusion_root),
            '--grpc_python_out={}'.format(inclusion_root),
            '--mypy_out={}'.format(inclusion_root),
        ] + [proto_file]
        if protoc.main(command) != 0:
            if strict_mode:
                raise Exception('error: {} failed'.format(command))
            else:
                sys.stderr.write('warning: {} failed'.format(command))


class BuildPackageProtos(Command):
    """Command to generate project *_pb2.py modules from proto files."""

    description = 'build grpc protobuf modules'
    user_options: List[Tuple[str, str, str]] = []

    def initialize_options(self):
        pass

    def finalize_options(self):
        pass

    def run(self):
        # import grpc_tools.command
        # grpc_tools.command.build_package_protos('.')
        build_package_protos('.')


setup(
    name="squeaknode",
    version=__version__,
    url="https://github.com/yzernik/squeaknode",
    packages=find_packages(),
    include_package_data=True,
    zip_safe=False,
    setup_requires=['grpcio-tools==1.39.0', 'wheel==0.37.1'],
    install_requires=requirements,
    extras_require={
        "test": ["pytest", "coverage"],
        "postgres": ["psycopg2"],
    },
    entry_points={
        'console_scripts': [
            'squeaknode = squeaknode.main:main',
        ],
    },
    cmdclass={
        'build_proto_modules': BuildPackageProtos,
        'build_py': BuildPyCommand,
    },
)
