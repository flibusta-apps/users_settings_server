from fastapi import APIRouter, HTTPException, status, Depends

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
from app.services import update_user_allowed_langs


# TODO: add redis cache


users_router = APIRouter(
    prefix="/users", tags=["users"], dependencies=[Depends(check_token)]
)


@users_router.get("/", dependencies=[Depends(Params)], response_model=Page[UserDetail])
async def get_users():
    return await paginate(User.objects.select_related("allowed_langs"))


@users_router.get("/{user_id}", response_model=UserDetail)
async def get_user(user_id: int):
    user_data = await User.objects.select_related("allowed_langs").get_or_none(
        user_id=user_id
    )

    if user_data is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND)

    return user_data


@users_router.post("/", response_model=UserDetail)
async def create_or_update_user(data: UserCreateOrUpdate):
    data_dict = data.dict()

    user_data = await User.objects.select_related("allowed_langs").get_or_none(
        user_id=data_dict["user_id"]
    )

    allowed_langs = data_dict.pop("allowed_langs")

    if user_data is None:
        user_data = await User.objects.select_related("allowed_langs").create(
            **data_dict
        )
        if allowed_langs is None:
            allowed_langs = ["ru", "be", "uk"]
    else:
        data_dict.pop("user_id")
        user_data.update_from_dict(data_dict)

    if allowed_langs:
        await update_user_allowed_langs(user_data, allowed_langs)

    return user_data


@users_router.patch("/{user_id}", response_model=UserDetail)
async def update_user(user_id: int, data: UserUpdate):
    user_data = await User.objects.select_related("allowed_langs").get_or_none(
        user_id=user_id
    )

    if user_data is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST)

    data_dict = data.dict()

    update_data = {}
    for key in data_dict:
        if data_dict[key] is not None:
            update_data[key] = data_dict[key]

    if not update_data:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST)

    allowed_langs = update_data.pop("allowed_langs", None)

    if update_data:
        user_data.update_from_dict(update_data)
        await user_data.update()

    if not allowed_langs:
        return user_data

    await update_user_allowed_langs(user_data, allowed_langs)

    return user_data


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


healthcheck_router = APIRouter(
    tags=["healthcheck"], dependencies=[Depends(check_token)]
)


@healthcheck_router.get("/healthcheck")
async def healthcheck():
    return "Ok!"
