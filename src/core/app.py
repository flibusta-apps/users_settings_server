from fastapi import FastAPI
from fastapi.responses import ORJSONResponse

from fastapi_pagination import add_pagination
from prometheus_fastapi_instrumentator import Instrumentator
from redis import asyncio as aioredis

from app.views import users_router, languages_router, healthcheck_router
from core.config import env_config
from core.db import database


def start_app() -> FastAPI:
    app = FastAPI(default_response_class=ORJSONResponse)

    app.include_router(users_router)
    app.include_router(languages_router)
    app.include_router(healthcheck_router)

    app.state.database = database

    app.state.redis = aioredis.Redis(
        host=env_config.REDIS_HOST,
        port=env_config.REDIS_PORT,
        db=env_config.REDIS_DB,
        password=env_config.REDIS_PASSWORD,
    )

    add_pagination(app)

    @app.on_event("startup")
    async def startup() -> None:
        database_ = app.state.database
        if not database_.is_connected:
            await database_.connect()

    @app.on_event("shutdown")
    async def shutdown() -> None:
        database_ = app.state.database
        if database_.is_connected:
            await database_.disconnect()

    Instrumentator().instrument(app).expose(app, include_in_schema=True)

    return app
