from .donate_notifications import donation_notifications_router
from .healthcheck import healthcheck_router
from .languages import languages_router
from .users import users_router


routers = [
    donation_notifications_router,
    healthcheck_router,
    languages_router,
    users_router,
]


__all__ = [
    "routers",
]
