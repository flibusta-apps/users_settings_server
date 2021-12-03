from pydantic import BaseModel, constr


class LanguageDetail(BaseModel):
    id: int
    label: constr(max_length=16)  # type: ignore
    code: constr(max_length=4)  # type: ignore


class UserBase(BaseModel):
    user_id: int
    last_name: constr(max_length=64)  # type: ignore
    first_name: constr(max_length=64)  # type: ignore
    username: constr(max_length=32)  # type: ignore


class UserCreateOrUpdate(BaseModel):
    allowed_langs: list[str]


class UserDetail(UserCreateOrUpdate):
    allowed_langs: list[LanguageDetail]
