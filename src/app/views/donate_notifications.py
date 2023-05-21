from datetime import datetime

from fastapi import APIRouter, Depends

from app.depends import check_token
from app.models import ChatDonateNotification


NOTIFICATION_DELTA_DAYS = 30


donation_notifications_router = APIRouter(
    prefix="/donate_notifications",
    tags=["donate_notifications"],
    dependencies=[Depends(check_token)],
)


@donation_notifications_router.get("/{chat_id}/is_need_send")
async def is_need_send(chat_id: int) -> bool:
    # add redis cache
    notification = await ChatDonateNotification.objects.get_or_none(chat_id=chat_id)

    if notification is None:
        return True

    delta = datetime.now() - notification.sended
    return delta.days >= NOTIFICATION_DELTA_DAYS


@donation_notifications_router.post("/{chat_id}")
async def mark_sended(chat_id: int):
    notification, created = await ChatDonateNotification.objects.get_or_create(
        _default={"sended": datetime.now()}, chat_id=chat_id
    )

    if created:
        return

    notification.sended = datetime.now()
    await notification.save()
