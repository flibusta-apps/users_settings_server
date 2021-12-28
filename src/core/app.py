from fastapi import FastAPI

from fastapi_pagination import add_pagination

from core.db import database
from app.views import users_router, languages_router


def start_app() -> FastAPI:
    app = FastAPI()

    app.include_router(users_router)
    app.include_router(languages_router)

    app.state.database = database

    add_pagination(app)

    @app.on_event('startup')
    async def startup() -> None:
        database_ = app.state.database
        if not database_.is_connected:
            await database_.connect()

    @app.on_event('shutdown')
    async def shutdown() -> None:
        database_ = app.state.database
        if database_.is_connected:
            await database_.disconnect()

    return app
