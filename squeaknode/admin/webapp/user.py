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

from flask_login import UserMixin
from werkzeug.security import check_password_hash
from werkzeug.security import generate_password_hash

logger = logging.getLogger(__name__)


class User(UserMixin):
    def __init__(
            self,
            username,
            password_hash,
    ):
        self.username = username
        self.password_hash = password_hash

    def is_authenticated(self):
        return True

    def is_active(self):
        return True

    def is_anonymous(self):
        return False

    def get_id(self):
        return self.username

    def check_password(self, password):
        return check_password_hash(self.password_hash, password)


class UserLookup:
    def __init__(
            self,
            admin_username,
            admin_password,
            handler,
    ):
        self.admin_username = admin_username
        self.admin_password = admin_password
        self.handler = handler

    def get_user_by_username(self, username):
        if self.admin_username == username:
            return User(
                self.admin_username,
                generate_password_hash(self.admin_password),
            )
        db_user = self.handler.handle_lookup_user(username)
        if db_user:
            return User(
                db_user.username,
                db_user.password_hash,
            )

    def register_user(self, username, password):
        if self.admin_username == username:
            raise Exception("Username belongs to admin.")
        try:
            user_id = self.handler.handle_register_user(
                username,
                generate_password_hash(password),
            )
            return user_id
        except Exception:
            return None
