import sentry_sdk
from sentry_sdk.integrations.asgi import SentryAsgiMiddleware

from core.app import start_app
from core.config import env_config

sentry_sdk.init(dsn=env_config.SENTRY_SDN)

app = SentryAsgiMiddleware(start_app())
