from typing import cast

from app.models import User, Language


async def update_user_allowed_langs(user: User, new_allowed_langs: list[str]) -> bool:
    user_allowed_langs = cast(list[Language], user.allowed_langs)

    exists_langs = set(lang.code for lang in user_allowed_langs)
    new_langs = set(new_allowed_langs)

    to_delete = exists_langs - new_langs
    to_add = new_langs - exists_langs

    all_process_langs = list(to_delete) + list(to_add)

    langs = await Language.objects.filter(code__in=all_process_langs).all()

    updated = False

    for lang in langs:
        if lang.code in to_delete:
            await user.allowed_langs.remove(lang)
            updated = True

        if lang.code in to_add:
            await user.allowed_langs.add(lang)
            updated = True

    return updated
