"""empty message

Revision ID: 64fe2045bf28
Revises: 750640043cd4
Create Date: 2023-01-05 18:28:05.296720

"""
from alembic import op
import sqlalchemy as sa


# revision identifiers, used by Alembic.
revision = "64fe2045bf28"
down_revision = "750640043cd4"
branch_labels = None
depends_on = None


def upgrade():
    # ### commands auto generated by Alembic - please adjust! ###
    op.create_table(
        "user_activity",
        sa.Column("id", sa.Integer(), nullable=False),
        sa.Column("user", sa.Integer(), nullable=False),
        sa.Column("updated", sa.DateTime(), nullable=False),
        sa.ForeignKeyConstraint(
            ["user"],
            ["user_settings.id"],
            name="fk_user_activity_user_settings_id_user",
        ),
        sa.PrimaryKeyConstraint("id"),
        sa.UniqueConstraint("user"),
    )
    # ### end Alembic commands ###


def downgrade():
    # ### commands auto generated by Alembic - please adjust! ###
    op.drop_table("user_activity")
    # ### end Alembic commands ###