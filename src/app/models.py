import ormar

from core.db import metadata, database


class BaseMeta(ormar.ModelMeta):
    metadata = metadata
    database = database


class Language(ormar.Model):
    class Meta(BaseMeta):
        tablename = "languages"

    id: int = ormar.Integer(primary_key=True)  # type: ignore
    label: str = ormar.String(max_length=16)  # type: ignore
    code: str = ormar.String(max_length=4, unique=True)  # type: ignore


class User(ormar.Model):
    class Meta(BaseMeta):
        tablename = "user_settings"

    id: int = ormar.Integer(primary_key=True)  # type: ignore

    user_id: int = ormar.BigInteger(unique=True)  # type: ignore
    last_name: str = ormar.String(max_length=64)  # type: ignore
    first_name: str = ormar.String(max_length=64)  # type: ignore
    username: str = ormar.String(max_length=32)  # type: ignore
    source: str = ormar.String(max_length=32)  # type: ignore

    allowed_langs = ormar.ManyToMany(Language)
