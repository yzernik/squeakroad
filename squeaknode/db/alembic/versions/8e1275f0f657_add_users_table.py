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
"""Add users table

Revision ID: 8e1275f0f657
Revises: b3d0395263c4
Create Date: 2022-06-09 21:07:46.344658

"""
import sqlalchemy as sa
from alembic import op

import squeaknode.db.models


# revision identifiers, used by Alembic.
revision = '8e1275f0f657'
down_revision = 'b3d0395263c4'
branch_labels = None
depends_on = None


def upgrade():
    op.create_table('user',
                    sa.Column('user_id', sa.Integer(), nullable=False),
                    sa.Column(
                        'created_time_ms', squeaknode.db.models.SLBigInteger(), nullable=False),
                    sa.Column('username', sa.String(), nullable=False),
                    sa.Column('password_hash', sa.String(), nullable=False),
                    sa.Column('user_image', sa.LargeBinary(), nullable=True),
                    sa.PrimaryKeyConstraint('user_id', name=op.f('pk_user')),
                    sa.UniqueConstraint(
                        'username', name=op.f('uq_user_username')),
                    sqlite_autoincrement=True
                    )


def downgrade():
    op.drop_table('user')
