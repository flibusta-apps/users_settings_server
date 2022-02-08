from typing import Optional, Union

from fastapi import HTTPException, status

import aioredis
import orjson

from app.models import User
from app.serializers import UserCreateOrUpdate, UserDetail, UserUpdate
from app.services.allowed_langs_updater import update_user_allowed_langs


class UsersDataManager:
    @classmethod
    async def _get_user_from_db(cls, user_id: int) -> Optional[User]:
        return await User.objects.select_related("allowed_langs").get_or_none(
            user_id=user_id
        )

    @classmethod
    def _get_cache_key(cls, user_id: int) -> str:
        return f"user_{user_id}"

    @classmethod
    async def _get_user_from_cache(
        cls, user_id: int, redis: aioredis.Redis
    ) -> Optional[UserDetail]:
        try:
            key = cls._get_cache_key(user_id)
            data = await redis.get(key)

            if data is None:
                return None

            return UserDetail.parse_obj(orjson.loads(data))

        except aioredis.RedisError:
            return None

    @classmethod
    async def _cache_user(cls, user: User, redis: aioredis.Redis) -> bool:
        try:
            key = cls._get_cache_key(user.id)
            data = orjson.dumps(user.dict())
            await redis.set(key, data)
            return True
        except aioredis.RedisError:
            return False

    @classmethod
    async def get_user(
        cls, user_id: int, redis: aioredis.Redis
    ) -> Optional[UserDetail]:
        if cached_user := await cls._get_user_from_cache(user_id, redis):
            return cached_user

        user = await cls._get_user_from_db(user_id)

        if not user:
            return None

        await cls._cache_user(user, redis)
        return user  # type: ignore

    @classmethod
    def _is_has_data_to_update(cls, new_user: UserUpdate) -> bool:
        data_dict = new_user.dict()

        update_data = {}
        for key in data_dict:
            if data_dict[key] is not None:
                update_data[key] = data_dict[key]

        return bool(update_data)

    @classmethod
    async def _create(cls, data: UserCreateOrUpdate):
        data_dict = data.dict()
        allowed_langs = data_dict.pop("allowed_langs", None) or ["ru", "be", "uk"]

        user_obj = await User.objects.select_related("allowed_langs").create(
            **data_dict
        )
        await update_user_allowed_langs(user_obj, allowed_langs)

        return user_obj

    @classmethod
    async def _update(
        cls, user_id: int, update_data: dict, redis: aioredis.Redis
    ) -> User:
        user_obj = await cls._get_user_from_db(user_id)
        assert user_obj is not None

        if allowed_langs := update_data.pop("allowed_langs", None):
            await update_user_allowed_langs(user_obj, allowed_langs)

        if update_data:
            user_obj.update_from_dict(update_data)
            await user_obj.update()

        await cls._cache_user(user_obj, redis)

        return user_obj

    @classmethod
    async def create_or_update_user(
        cls, data: UserCreateOrUpdate, redis: aioredis.Redis
    ):
        user = await cls.get_user(data.user_id, redis)

        if user is None:
            new_user = await cls._create(data)
            await cls._cache_user(new_user, redis)
            return new_user

        if not cls._is_need_update(user, data):
            return user

        return await cls._update(user.user_id, data.dict(), redis)

    @classmethod
    def _is_need_update(
        cls, old_user: UserDetail, new_user: Union[UserUpdate, UserCreateOrUpdate]
    ) -> bool:
        old_data = old_user.dict()
        new_data = new_user.dict()

        allowed_langs = new_data.pop("allowed_lang", None)

        for key in new_data:
            if new_data[key] != old_data[key]:
                return True

        if allowed_langs and set(allowed_langs) != set(
            [lang.code for lang in old_user.allowed_langs]
        ):
            return True

        return False

    @classmethod
    async def update_user(
        cls, user_id: int, user_data: UserUpdate, redis: aioredis.Redis
    ) -> Union[UserDetail, User]:
        user = await cls.get_user(user_id, redis)

        if user is None:
            raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST)

        if not cls._is_has_data_to_update(user_data):
            raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST)

        if not cls._is_need_update(user, user_data):
            return user

        return await cls._update(user.user_id, user_data.dict(), redis)
