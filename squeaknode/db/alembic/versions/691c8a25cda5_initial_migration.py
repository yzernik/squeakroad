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
"""Initial migration

Revision ID: 691c8a25cda5
Revises:
Create Date: 2022-02-27 21:10:50.785638

"""
import sqlalchemy as sa
from alembic import op

import squeaknode.db.models


# revision identifiers, used by Alembic.
revision = '691c8a25cda5'
down_revision = None
branch_labels = None
depends_on = None


def upgrade():
    op.create_table('config',
                    sa.Column('username', sa.String(), nullable=False),
                    sa.Column('sell_price_msat', sa.Integer(), nullable=True),
                    sa.PrimaryKeyConstraint('username', name=op.f('pk_config'))
                    )
    op.create_table('peer',
                    sa.Column('peer_id', sa.Integer(), nullable=False),
                    sa.Column(
                        'created_time_ms', squeaknode.db.models.SLBigInteger(), nullable=False),
                    sa.Column('peer_name', sa.String(), nullable=False),
                    sa.Column('network', sa.String(length=10), nullable=False),
                    sa.Column('host', sa.String(), nullable=False),
                    sa.Column('port', sa.Integer(), nullable=False),
                    sa.Column('autoconnect', sa.Boolean(), nullable=False),
                    sa.Column('share_for_free', sa.Boolean(), nullable=False),
                    sa.PrimaryKeyConstraint('peer_id', name=op.f('pk_peer')),
                    sa.UniqueConstraint(
                        'host', 'port', name='uq_peer_host_port'),
                    sqlite_autoincrement=True
                    )
    op.create_table('profile',
                    sa.Column('profile_id', sa.Integer(), nullable=False),
                    sa.Column(
                        'created_time_ms', squeaknode.db.models.SLBigInteger(), nullable=False),
                    sa.Column('profile_name', sa.String(), nullable=False),
                    sa.Column('private_key', sa.LargeBinary(), nullable=True),
                    sa.Column('public_key', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('following', sa.Boolean(), nullable=False),
                    sa.Column('profile_image',
                              sa.LargeBinary(), nullable=True),
                    sa.PrimaryKeyConstraint(
                        'profile_id', name=op.f('pk_profile')),
                    sa.UniqueConstraint('profile_name', name=op.f(
                        'uq_profile_profile_name')),
                    sa.UniqueConstraint(
                        'public_key', name=op.f('uq_profile_public_key')),
                    sqlite_autoincrement=True
                    )
    op.create_table('received_offer',
                    sa.Column(
                        'received_offer_id', squeaknode.db.models.SLBigInteger(), nullable=False),
                    sa.Column(
                        'created_time_ms', squeaknode.db.models.SLBigInteger(), nullable=False),
                    sa.Column('squeak_hash', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('payment_hash', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('nonce', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('payment_point', sa.LargeBinary(
                        length=33), nullable=False),
                    sa.Column('invoice_timestamp',
                              sa.Integer(), nullable=False),
                    sa.Column('invoice_expiry', sa.Integer(), nullable=False),
                    sa.Column('price_msat', sa.Integer(), nullable=False),
                    sa.Column('payment_request', sa.String(), nullable=False),
                    sa.Column('destination', sa.String(
                        length=66), nullable=False),
                    sa.Column('lightning_host', sa.String(), nullable=False),
                    sa.Column('lightning_port', sa.Integer(), nullable=False),
                    sa.Column('peer_network', sa.String(
                        length=10), nullable=False),
                    sa.Column('peer_host', sa.String(), nullable=False),
                    sa.Column('peer_port', sa.Integer(), nullable=False),
                    sa.Column('paid', sa.Boolean(), nullable=False),
                    sa.PrimaryKeyConstraint('received_offer_id',
                                            name=op.f('pk_received_offer')),
                    sa.UniqueConstraint('payment_hash', name=op.f(
                        'uq_received_offer_payment_hash')),
                    sqlite_autoincrement=True
                    )
    op.create_table('received_payment',
                    sa.Column('received_payment_id',
                              squeaknode.db.models.SLBigInteger(), nullable=False),
                    sa.Column(
                        'created_time_ms', squeaknode.db.models.SLBigInteger(), nullable=False),
                    sa.Column('squeak_hash', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('payment_hash', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('price_msat', sa.Integer(), nullable=False),
                    sa.Column('settle_index',
                              squeaknode.db.models.SLBigInteger(), nullable=False),
                    sa.Column('peer_network', sa.String(
                        length=10), nullable=False),
                    sa.Column('peer_host', sa.String(), nullable=False),
                    sa.Column('peer_port', sa.Integer(), nullable=False),
                    sa.PrimaryKeyConstraint('received_payment_id',
                                            name=op.f('pk_received_payment')),
                    sa.UniqueConstraint('payment_hash', name=op.f(
                        'uq_received_payment_payment_hash')),
                    sqlite_autoincrement=True
                    )
    op.create_table('sent_offer',
                    sa.Column(
                        'sent_offer_id', squeaknode.db.models.SLBigInteger(), nullable=False),
                    sa.Column(
                        'created_time_ms', squeaknode.db.models.SLBigInteger(), nullable=False),
                    sa.Column('squeak_hash', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('payment_hash', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('nonce', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('price_msat', sa.Integer(), nullable=False),
                    sa.Column('payment_request', sa.String(), nullable=False),
                    sa.Column('invoice_timestamp',
                              sa.Integer(), nullable=False),
                    sa.Column('invoice_expiry', sa.Integer(), nullable=False),
                    sa.Column('peer_network', sa.String(
                        length=10), nullable=False),
                    sa.Column('peer_host', sa.String(), nullable=False),
                    sa.Column('peer_port', sa.Integer(), nullable=False),
                    sa.Column('paid', sa.Boolean(), nullable=False),
                    sa.PrimaryKeyConstraint(
                        'sent_offer_id', name=op.f('pk_sent_offer')),
                    sa.UniqueConstraint('payment_hash', name=op.f(
                        'uq_sent_offer_payment_hash')),
                    sqlite_autoincrement=True
                    )
    op.create_table('sent_payment',
                    sa.Column(
                        'sent_payment_id', squeaknode.db.models.SLBigInteger(), nullable=False),
                    sa.Column(
                        'created_time_ms', squeaknode.db.models.SLBigInteger(), nullable=False),
                    sa.Column('peer_network', sa.String(
                        length=10), nullable=False),
                    sa.Column('peer_host', sa.String(), nullable=False),
                    sa.Column('peer_port', sa.Integer(), nullable=False),
                    sa.Column('squeak_hash', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('payment_hash', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('secret_key', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('price_msat', sa.Integer(), nullable=False),
                    sa.Column('node_pubkey', sa.String(
                        length=66), nullable=False),
                    sa.Column('valid', sa.Boolean(), nullable=False),
                    sa.PrimaryKeyConstraint(
                        'sent_payment_id', name=op.f('pk_sent_payment')),
                    sa.UniqueConstraint('payment_hash', name=op.f(
                        'uq_sent_payment_payment_hash')),
                    sqlite_autoincrement=True
                    )
    op.create_table('squeak',
                    sa.Column('hash', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column(
                        'created_time_ms', squeaknode.db.models.SLBigInteger(), nullable=False),
                    sa.Column('squeak', sa.LargeBinary(), nullable=False),
                    sa.Column('reply_hash', sa.LargeBinary(
                        length=32), nullable=True),
                    sa.Column('block_hash', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('block_height', sa.Integer(), nullable=False),
                    sa.Column('time_s', sa.Integer(), nullable=False),
                    sa.Column('author_public_key', sa.LargeBinary(
                        length=32), nullable=False),
                    sa.Column('recipient_public_key', sa.LargeBinary(
                        length=32), nullable=True),
                    sa.Column('secret_key', sa.LargeBinary(
                        length=32), nullable=True),
                    sa.Column('block_time_s', sa.Integer(), nullable=False),
                    sa.Column('liked_time_ms',
                              squeaknode.db.models.SLBigInteger(), nullable=True),
                    sa.Column('content', sa.String(length=280), nullable=True),
                    sa.PrimaryKeyConstraint('hash', name=op.f('pk_squeak'))
                    )
    with op.batch_alter_table('squeak', schema=None) as batch_op:
        batch_op.create_index(batch_op.f('ix_squeak_author_public_key'), [
                              'author_public_key'], unique=False)
        batch_op.create_index(batch_op.f('ix_squeak_recipient_public_key'), [
                              'recipient_public_key'], unique=False)

    op.create_table('twitter_account',
                    sa.Column('twitter_account_id',
                              sa.Integer(), nullable=False),
                    sa.Column('handle', sa.String(), nullable=False),
                    sa.Column('profile_id', sa.Integer(), nullable=False),
                    sa.Column('bearer_token', sa.String(), nullable=False),
                    sa.PrimaryKeyConstraint('twitter_account_id',
                                            name=op.f('pk_twitter_account')),
                    sa.UniqueConstraint('handle', name=op.f(
                        'uq_twitter_account_handle'))
                    )


def downgrade():
    op.drop_table('twitter_account')
    with op.batch_alter_table('squeak', schema=None) as batch_op:
        batch_op.drop_index(batch_op.f('ix_squeak_recipient_public_key'))
        batch_op.drop_index(batch_op.f('ix_squeak_author_public_key'))

    op.drop_table('squeak')
    op.drop_table('sent_payment')
    op.drop_table('sent_offer')
    op.drop_table('received_payment')
    op.drop_table('received_offer')
    op.drop_table('profile')
    op.drop_table('peer')
    op.drop_table('config')
