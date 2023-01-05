from typing import Optional

from pydantic import BaseModel, constr


class CreateLanguage(BaseModel):
    label: constr(max_length=16)  # type: ignore
    code: constr(max_length=4)  # type: ignore


class LanguageDetail(CreateLanguage):
    id: int


class UserBase(BaseModel):
    user_id: int
    last_name: constr(max_length=64)  # type: ignore
    first_name: constr(max_length=64)  # type: ignore
    username: constr(max_length=32)  # type: ignore
    source: constr(max_length=32)  # type: ignore


class UserCreateOrUpdate(UserBase):
    allowed_langs: Optional[list[str]] = None


class UserUpdate(BaseModel):
    last_name: Optional[constr(max_length=64)] = None  # type: ignore
    first_name: Optional[constr(max_length=64)] = None  # type: ignore
    username: Optional[constr(max_length=32)] = None  # type: ignore
    source: Optional[constr(max_length=32)] = None  # type: ignore
    allowed_langs: Optional[list[str]] = None


class UserDetail(UserBase):
    id: int
    allowed_langs: list[LanguageDetail]
