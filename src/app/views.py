from fastapi import APIRouter, HTTPException, status, Depends, Request

import aioredis
from fastapi_pagination import Page, Params
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token
from app.models import User, Language
from app.serializers import (
    UserCreateOrUpdate,
    UserUpdate,
    UserDetail,
    CreateLanguage,
    LanguageDetail,
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


languages_router = APIRouter(
    prefix="/languages", tags=["languages"], dependencies=[Depends(check_token)]
)


@languages_router.get("/", response_model=list[LanguageDetail])
async def get_languages():
    return await Language.objects.all()


@languages_router.get("/{code}", response_model=LanguageDetail)
async def get_language(code: str):
    language = await Language.objects.get_or_none(code=code)

    if language is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND)

    return language


@languages_router.post("/", response_model=LanguageDetail)
async def create_language(data: CreateLanguage):
    return await Language.objects.create(**data.dict())


healthcheck_router = APIRouter(tags=["healthcheck"])


@healthcheck_router.get("/healthcheck")
async def healthcheck():
    return "Ok!"
