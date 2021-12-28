from pydantic import BaseModel, constr


class CreateLanguage(BaseModel):
    label: constr(max_length=16)  # type: ignore
    code: constr(max_length=4)  # type: ignore


class LanguageDetail(CreateLanguage):
    id: int


class UserBase(BaseModel):
    last_name: constr(max_length=64)  # type: ignore
    first_name: constr(max_length=64)  # type: ignore
    username: constr(max_length=32)  # type: ignore
    source: constr(max_length=32)  # type: ignore


class UserCreateOrUpdate(UserBase):
    allowed_langs: list[str]


class UserDetail(BaseModel):
    user_id: int
    allowed_langs: list[LanguageDetail]
