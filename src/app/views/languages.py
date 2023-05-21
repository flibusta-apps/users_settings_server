from fastapi import APIRouter, Depends, HTTPException, status

from app.depends import check_token
from app.models import Language
from app.serializers import (
    CreateLanguage,
    LanguageDetail,
)


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
