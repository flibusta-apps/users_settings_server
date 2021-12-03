from fastapi import APIRouter, HTTPException, status

from app.serializers import UserCreateOrUpdate, UserDetail, LanguageDetail
from app.models import User, Language


# TODO: add redis cache


users_router = APIRouter(
    prefix="/users",
    tags=["users"]
)


@users_router.get("/", response_model=list[UserDetail])
async def get_users():
    return await User.objects.select_related("allowed_langs").all()


@users_router.get("/{user_id}", response_model=UserDetail)
async def get_user(user_id: int):
    user_data = await User.objects.select_related("allowd_langs").get_or_none(user_id=user_id)

    if user_data is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND)

    return user_data


@users_router.post("/{user_id}", response_model=UserDetail)
async def get_or_update_user(user_id: int, data: UserCreateOrUpdate):
    data_dict = data.dict()

    user_data = await User.objects.select_related("allowed_langs").get_or_none(user_id=user_id)

    allowed_langs = data_dict.pop("allowed_langs")

    if user_data is None:
        return User.objects.create(**data.dict())
    else:
        user_data.update_from_dict(data.dict())

    user_data.allowed_langs.clear()

    langs = await Language.objects.filter(code__in=allowed_langs).all()

    for lang in langs:
        await user_data.allowed_langs.add(lang)

    return user_data


languages_router = APIRouter(
    prefix="/languages",
    tags=["languages"]
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
