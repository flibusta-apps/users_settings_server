from datetime import datetime

from fastapi import APIRouter, Depends, HTTPException, Request, status

from asyncpg.exceptions import UniqueViolationError
from fastapi_pagination import Page, Params
from fastapi_pagination.ext.ormar import paginate
from redis import asyncio as aioredis

from app.depends import check_token
from app.models import User, UserActivity
from app.serializers import (
    UserCreateOrUpdate,
    UserDetail,
    UserUpdate,
)
from app.services.users_data_manager import UsersDataManager


users_router = APIRouter(
    prefix="/users", tags=["users"], dependencies=[Depends(check_token)]
)


@users_router.get("/", dependencies=[Depends(Params)], response_model=Page[UserDetail])
async def get_users():
    return await paginate(User.objects.select_related("allowed_langs"))


@users_router.get("/{user_id}", response_model=UserDetail)
async def get_user(request: Request, user_id: int):
    redis: aioredis.Redis = request.app.state.redis
    user_data = await UsersDataManager.get_user(user_id, redis)

    if user_data is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND)

    return user_data


@users_router.post("/", response_model=UserDetail)
async def create_or_update_user(request: Request, data: UserCreateOrUpdate):
    redis: aioredis.Redis = request.app.state.redis
    return await UsersDataManager.create_or_update_user(data, redis)


@users_router.patch("/{user_id}", response_model=UserDetail)
async def update_user(request: Request, user_id: int, data: UserUpdate):
    redis: aioredis.Redis = request.app.state.redis
    return await UsersDataManager.update_user(user_id, data, redis)


@users_router.post("/{user_id}/update_activity")
async def update_activity(user_id: int) -> None:
    user = await User.objects.get_or_none(user_id=user_id)

    if user is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND)

    activity = await UserActivity.objects.get_or_none(user__user_id=user_id)

    if activity is None:
        try:
            await UserActivity.objects.create(user=user.id, updated=datetime.now())
        except UniqueViolationError:
            pass
        return

    activity.updated = datetime.now()
    await activity.update()
