use std::collections::HashMap;

use crate::{
    prisma::{language, language_to_user, user_settings},
    views::Database,
};

pub async fn update_languages(user: user_settings::Data, new_langs: Vec<String>, db: Database) {
    // Delete
    {
        let need_delete: Vec<_> = user
            .languages()
            .unwrap()
            .iter()
            .map(|item| {
                let language::Data { id, code, .. } = *item.clone().language.unwrap();
                (id, code)
            })
            .filter(|(_, code)| !new_langs.contains(code))
            .map(|(id, _)| id)
            .collect();

        let _ = db
            .language_to_user()
            .delete_many(vec![language_to_user::id::in_vec(need_delete)])
            .exec()
            .await;
    }

    // Create
    {
        let languages: HashMap<_, _> = db
            .language()
            .find_many(vec![])
            .exec()
            .await
            .unwrap()
            .into_iter()
            .map(|l| (l.code, l.id))
            .collect();

        let current_langs: Vec<_> = user
            .languages()
            .unwrap()
            .iter()
            .map(|item| item.clone().language.unwrap().code)
            .collect();

        let need_create: Vec<i32> = new_langs
            .into_iter()
            .filter(|code| !current_langs.contains(code))
            .map(|code| *languages.get(&code).unwrap())
            .collect();

        let _ = db
            .language_to_user()
            .create_many(
                need_create
                    .iter()
                    .map(|language_id| {
                        language_to_user::create_unchecked(*language_id, user.id, vec![])
                    })
                    .collect(),
            )
            .exec()
            .await;
    }
}
