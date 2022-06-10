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
"""Allow peer name to be null

Revision ID: a2122c4711cc
Revises: 691c8a25cda5
Create Date: 2022-03-15 14:31:38.050091

"""
import sqlalchemy as sa
from alembic import op


# revision identifiers, used by Alembic.
revision = 'a2122c4711cc'
down_revision = '691c8a25cda5'
branch_labels = None
depends_on = None


def upgrade():
    with op.batch_alter_table('peer', schema=None) as batch_op:
        batch_op.alter_column('peer_name',
                              existing_type=sa.VARCHAR(),
                              nullable=True)


def downgrade():
    with op.batch_alter_table('peer', schema=None) as batch_op:
        batch_op.alter_column('peer_name',
                              existing_type=sa.VARCHAR(),
                              nullable=False)
