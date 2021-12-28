from fastapi import APIRouter, HTTPException, status, Depends

from fastapi_pagination import Page, Params
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token
from app.serializers import UserCreateOrUpdate, UserDetail, CreateLanguage, LanguageDetail
from app.models import User, Language


# TODO: add redis cache


users_router = APIRouter(
    prefix="/users",
    tags=["users"],
    dependencies=[Depends(check_token)]
)


@users_router.get("/", dependencies=[Depends(Params)], response_model=Page[UserDetail])
async def get_users():
    return await paginate(User.objects.select_related("allowed_langs"))


@users_router.get("/{user_id}", response_model=UserDetail)
async def get_user(user_id: int):
    user_data = await User.objects.select_related("allowd_langs").get_or_none(user_id=user_id)

    if user_data is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND)

    return user_data


@users_router.post("/{user_id}", response_model=UserDetail)
async def create_or_update_user(user_id: int, data: UserCreateOrUpdate):
    data_dict = data.dict()

    user_data = await User.objects.select_related("allowed_langs").get_or_none(user_id=user_id)

    allowed_langs = data_dict.pop("allowed_langs")

    if user_data is None:
        user_data = await User.objects.select_related("allowed_langs").create(**{**data_dict, "user_id": user_id})
    else:
        user_data.update_from_dict(data_dict)

    await user_data.allowed_langs.clear()  # type: ignore

    langs = await Language.objects.filter(code__in=allowed_langs).all()

    for lang in langs:
        await user_data.allowed_langs.add(lang)
    
    await user_data.update()

    return user_data


languages_router = APIRouter(
    prefix="/languages",
    tags=["languages"],
    dependencies=[Depends(check_token)]
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
