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
"""Remove block hash and block height

Revision ID: 305c2464f89f
Revises: 8e1275f0f657
Create Date: 2022-06-11 17:11:50.562106

"""
import sqlalchemy as sa
from alembic import op


# revision identifiers, used by Alembic.
revision = '305c2464f89f'
down_revision = '8e1275f0f657'
branch_labels = None
depends_on = None


def upgrade():
    with op.batch_alter_table('squeak', schema=None) as batch_op:
        batch_op.drop_column('block_height')
        batch_op.drop_column('block_time_s')
        batch_op.drop_column('block_hash')


def downgrade():
    with op.batch_alter_table('squeak', schema=None) as batch_op:
        batch_op.add_column(sa.Column('block_hash', sa.BLOB(), nullable=False))
        batch_op.add_column(
            sa.Column('block_time_s', sa.INTEGER(), nullable=False))
        batch_op.add_column(
            sa.Column('block_height', sa.INTEGER(), nullable=False))
