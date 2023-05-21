from fastapi import APIRouter


healthcheck_router = APIRouter(tags=["healthcheck"])


@healthcheck_router.get("/healthcheck")
async def healthcheck():
    return "Ok!"
