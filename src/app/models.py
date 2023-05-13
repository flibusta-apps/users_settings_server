from datetime import datetime
from typing import cast

import ormar

from core.db import database, metadata


class BaseMeta(ormar.ModelMeta):
    metadata = metadata
    database = database


class Language(ormar.Model):
    class Meta(BaseMeta):
        tablename = "languages"

    id: int = cast(int, ormar.Integer(primary_key=True))
    label: str = cast(str, ormar.String(max_length=16))
    code: str = cast(str, ormar.String(max_length=4, unique=True))


class User(ormar.Model):
    class Meta(BaseMeta):
        tablename = "user_settings"

    id: int = cast(int, ormar.Integer(primary_key=True))

    user_id: int = cast(int, ormar.BigInteger(unique=True))
    last_name: str = cast(str, ormar.String(max_length=64))
    first_name: str = cast(str, ormar.String(max_length=64))
    username: str = cast(str, ormar.String(max_length=32))
    source: str = cast(str, ormar.String(max_length=32))

    allowed_langs = ormar.ManyToMany(Language)


class UserActivity(ormar.Model):
    class Meta(BaseMeta):
        tablename = "user_activity"

    id: int = cast(int, ormar.Integer(primary_key=True))

    user: User = ormar.ForeignKey(
        User, nullable=False, unique=True, related_name="last_activity"
    )
    updated: datetime = cast(datetime, ormar.DateTime(timezone=False))


class ChatDonateNotification(ormar.Model):
    class Meta(BaseMeta):
        tablename = "chat_donate_notifications"

    id: int = cast(int, ormar.BigInteger(primary_key=True))
    chat_id: int = cast(int, ormar.BigInteger(unique=True))
    sended: datetime = cast(datetime, ormar.DateTime(timezone=False))
