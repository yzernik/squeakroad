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
import uuid

import grpc
import pytest
from squeak.params import SelectParams

from proto import squeak_admin_pb2
from proto import squeak_admin_pb2_grpc
from tests.util import bytes_to_base64_string
from tests.util import create_contact_profile
from tests.util import create_saved_peer
from tests.util import create_signing_profile
from tests.util import delete_profile
from tests.util import generate_private_key
from tests.util import get_public_key
from tests.util import saved_peer


@pytest.fixture(autouse=True)
def select_mainnet_params():
    # Set the network to simnet
    SelectParams("simnet")


@pytest.fixture
def admin_stub():
    with grpc.insecure_channel("squeaknode:8994") as admin_channel:
        yield squeak_admin_pb2_grpc.SqueakAdminStub(admin_channel)


@pytest.fixture
def other_admin_stub():
    with grpc.insecure_channel("squeaknode_other:8994") as admin_channel:
        yield squeak_admin_pb2_grpc.SqueakAdminStub(admin_channel)


@pytest.fixture
def admin_peer(other_admin_stub):
    with saved_peer(other_admin_stub, 'squeaknode', 'squeaknode', 18777):
        yield


@pytest.fixture
def private_key():
    # Create a private key
    yield generate_private_key()


@pytest.fixture
def public_key(private_key):
    yield get_public_key(private_key)


# @pytest.fixture
# def profile_name():
#     yield "fake_profile_{}".format(uuid.uuid1())


@pytest.fixture
def signing_profile_id(admin_stub, random_name):
    # Create a new signing profile
    profile_id = create_signing_profile(admin_stub, random_name)
    yield profile_id
    # Delete the profile
    delete_profile(admin_stub, profile_id)


@pytest.fixture
def contact_profile_id(admin_stub, random_name, public_key):
    # Create a new contact profile
    contact_profile_id = create_contact_profile(
        admin_stub, random_name, public_key)
    yield contact_profile_id
    # Delete the profile
    admin_stub.DeleteSqueakProfile(
        squeak_admin_pb2.DeleteSqueakProfileRequest(
            profile_id=contact_profile_id,
        )
    )


@pytest.fixture
def recipient_signing_profile_id(admin_stub, random_recipient_name):
    # Create a new signing profile
    profile_id = create_signing_profile(admin_stub, random_recipient_name)
    yield profile_id
    # Delete the profile
    delete_profile(admin_stub, profile_id)


@pytest.fixture
def recipient_contact_profile_id(admin_stub, random_recipient_name, public_key):
    # Create a new contact profile
    contact_profile_id = create_contact_profile(
        admin_stub, random_recipient_name, public_key)
    yield contact_profile_id
    # Delete the profile
    admin_stub.DeleteSqueakProfile(
        squeak_admin_pb2.DeleteSqueakProfileRequest(
            profile_id=contact_profile_id,
        )
    )


@pytest.fixture
def saved_squeak_hash(admin_stub, signing_profile_id):
    # Create a new squeak using the new profile
    make_squeak_content = "Hello from the profile on the server!"
    make_squeak_response = admin_stub.MakeSqueak(
        squeak_admin_pb2.MakeSqueakRequest(
            profile_id=signing_profile_id,
            content=make_squeak_content,
        )
    )
    squeak_hash = make_squeak_response.squeak_hash
    yield squeak_hash
    # Delete the squeak
    admin_stub.DeleteSqueak(
        squeak_admin_pb2.DeleteSqueakRequest(
            squeak_hash=squeak_hash,
        )
    )


@pytest.fixture
def peer_id(admin_stub, random_name):
    # Create a new peer
    peer_id = create_saved_peer(
        admin_stub,
        random_name,
        random_name,
        1234,
    )
    yield peer_id
    # Delete the peer
    admin_stub.DeletePeer(
        squeak_admin_pb2.DeletePeerRequest(
            peer_id=peer_id,
        )
    )


@pytest.fixture
def random_name():
    yield "random_name_{}".format(uuid.uuid1())


@pytest.fixture
def random_recipient_name():
    yield "random_recipient_name_{}".format(uuid.uuid1())


@pytest.fixture
def random_image():
    yield os.urandom(567)


@pytest.fixture
def random_image_base64_string(random_image):
    yield bytes_to_base64_string(random_image)
