from typing import Optional

from pydantic import BaseSettings


class EnvConfig(BaseSettings):
    API_KEY: str

    POSTGRES_USER: str
    POSTGRES_PASSWORD: str
    POSTGRES_HOST: str
    POSTGRES_PORT: int
    POSTGRES_DB: str

    REDIS_HOST: str
    REDIS_PORT: int
    REDIS_DB: int
    REDIS_PASSWORD: Optional[str]

    SENTRY_SDN: str


env_config = EnvConfig()
