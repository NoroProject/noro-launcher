//! Тексты по умолчанию для шаблонов наказаний.
//!
//! Вынесены из `moderation_messages`: это семнадцать длинных строк на каждый
//! язык, рядом с которыми логика загрузки просто не читается.
//!
//! Русский набор — не перевод английского, а первоисточник: именно он лежал в
//! `Default` до появления словаря по языкам, и именно на него ссылается
//! миграция, переносящая старое плоское значение.

use super::moderation_messages::ModerationMessages;

impl ModerationMessages {
    pub fn default_ru() -> Self {
        Self {
            ban_permanent: "#f87171&lДОСТУП В СЕТЬ ЗАБЛОКИРОВАН #94a3b8(Навсегда)\n\n#94a3b8Причина: #f8fafc{reason}\n#94a3b8Модератор: #f8fafc{actor}\n#94a3b8Правило: #f8fafc{rule_link} #94a3b8{rule_title}\n#94a3b8Дело: #64748b{id}\n\n#64748bАпелляция: https://noro.dalynkaa.dev/support".into(),
            ban_temporary: "#f87171&lДОСТУП В СЕТЬ ОГРАНИЧЕН\n\n#94a3b8Причина: #f8fafc{reason}\n#94a3b8Модератор: #f8fafc{actor}\n#94a3b8Окончание: #fbbf24{expires} #94a3b8(через #fbbf24{duration}#94a3b8)\n#94a3b8Правило: #f8fafc{rule_link} #94a3b8{rule_title}\n#94a3b8Дело: #64748b{id}\n\n#64748bАпелляция: https://noro.dalynkaa.dev/support".into(),
            server_ban_permanent: "#f87171&lДОСТУП К СЕРВЕРУ ЗАБЛОКИРОВАН #94a3b8(Навсегда)\n\n#94a3b8Причина: #f8fafc{reason}\n#94a3b8Модератор: #f8fafc{actor}\n#94a3b8Правило: #f8fafc{rule_link} #94a3b8{rule_title}\n#94a3b8Дело: #64748b{id}\n\n#64748bВы можете играть на других серверах сети.".into(),
            server_ban_temporary: "#f87171&lДОСТУП К СЕРВЕРУ ВРЕМЕННО ОГРАНИЧЕН\n\n#94a3b8Причина: #f8fafc{reason}\n#94a3b8Модератор: #f8fafc{actor}\n#94a3b8Окончание: #fbbf24{expires} #94a3b8(через #fbbf24{duration}#94a3b8)\n#94a3b8Правило: #f8fafc{rule_link} #94a3b8{rule_title}\n#94a3b8Дело: #64748b{id}\n\n#94a3b8На остальных серверах сети доступ сохранен.".into(),
            mute_permanent: "#f87171&l[!] #f87171Ваш чат заблокирован навсегда. #94a3b8Причина: #f8fafc{reason} #94a3b8(Выдал: #f8fafc{actor}#94a3b8)".into(),
            mute_temporary: "#f87171&l[!] #f87171Ваш чат заблокирован еще на #fbbf24{duration}#f87171. #94a3b8Причина: #f8fafc{reason} #94a3b8(Выдал: #f8fafc{actor}#94a3b8)".into(),
            mute_actionbar_permanent: "#f87171Чат заблокирован #94a3b8• #f8fafc{reason}".into(),
            mute_actionbar_temporary: "#f87171Чат заблокирован ещё на #fbbf24{duration} #94a3b8• #f8fafc{reason}".into(),
            warn_actionbar: "#fbbf24Предупреждение #94a3b8• #f8fafc{reason}".into(),
            warn_notice: "#fbbf24&l[!] #fbbf24Вам выдано предупреждение от #f8fafc{actor}#fbbf24. #94a3b8Причина: #f8fafc{reason} #94a3b8({rule_link})".into(),
            broadcast: "#f87171&l[Модерация] #f8fafc{player} #94a3b8получил наказание (#f87171{kind}#94a3b8) от #f8fafc{actor}#94a3b8: #f8fafc{reason}".into(),
            actor_receipt: "#4ade80&l[Успешно] #f8fafc{player} #94a3b8наказан (#4ade80{kind}#94a3b8): #f8fafc{reason}".into(),
            reason_by_rule: "Нарушение п.{rule}: {rule_title}".into(),
            no_account: "#fbbf24&lНУЖЕН АККАУНТ СЕТИ\n\n#94a3b8Мы вас не знаем: этот ник не привязан ни к одному аккаунту.\n#94a3b8Войдите через лаунчер — он заведёт аккаунт и вернёт вас сюда.\n\n#64748bhttps://noro.dalynkaa.dev".into(),
            no_access: "#fbbf24&lСЕРВЕР ЗАКРЫТ ДЛЯ ВАС\n\n#94a3b8У вашего аккаунта нет доступа к этой сборке.\n#94a3b8Если это ошибка — напишите в поддержку.\n\n#64748bhttps://noro.dalynkaa.dev/support".into(),
            maintenance: "#fbbf24&lТЕХНИЧЕСКОЕ ОБСЛУЖИВАНИЕ\n\n#94a3b8Сервер временно недоступен: ведутся технические работы.\n\n#64748bПопробуйте зайти позже.".into(),
        }
    }

    pub fn default_en() -> Self {
        Self {
            ban_permanent: "#f87171&lNETWORK ACCESS BLOCKED #94a3b8(Permanent)\n\n#94a3b8Reason: #f8fafc{reason}\n#94a3b8Moderator: #f8fafc{actor}\n#94a3b8Rule: #f8fafc{rule_link} #94a3b8{rule_title}\n#94a3b8Case: #64748b{id}\n\n#94a3b8Appeal: https://noro.dalynkaa.dev/support".into(),
            ban_temporary: "#f87171&lNETWORK ACCESS RESTRICTED\n\n#94a3b8Reason: #f8fafc{reason}\n#94a3b8Moderator: #f8fafc{actor}\n#94a3b8Expires: #fbbf24{expires} #94a3b8(in #fbbf24{duration}#94a3b8)\n#94a3b8Rule: #f8fafc{rule_link} #94a3b8{rule_title}\n#94a3b8Case: #64748b{id}\n\n#94a3b8Appeal: https://noro.dalynkaa.dev/support".into(),
            server_ban_permanent: "#f87171&lSERVER ACCESS BLOCKED #94a3b8(Permanent)\n\n#94a3b8Reason: #f8fafc{reason}\n#94a3b8Moderator: #f8fafc{actor}\n#94a3b8Rule: #f8fafc{rule_link} #94a3b8{rule_title}\n#94a3b8Case: #64748b{id}\n\n#94a3b8You can play on other network servers.".into(),
            server_ban_temporary: "#f87171&lSERVER ACCESS TEMPORARILY RESTRICTED\n\n#94a3b8Reason: #f8fafc{reason}\n#94a3b8Moderator: #f8fafc{actor}\n#94a3b8Expires: #fbbf24{expires} #94a3b8(in #fbbf24{duration}#94a3b8)\n#94a3b8Rule: #f8fafc{rule_link} #94a3b8{rule_title}\n#94a3b8Case: #64748b{id}\n\n#94a3b8Access to other network servers remains.".into(),
            mute_permanent: "#f87171&l[!] #f87171Your chat is muted permanently. #94a3b8Reason: #f8fafc{reason} #94a3b8(By: #f8fafc{actor}#94a3b8)".into(),
            mute_temporary: "#f87171&l[!] #f87171Your chat is muted for #fbbf24{duration}#f87171. #94a3b8Reason: #f8fafc{reason} #94a3b8(By: #f8fafc{actor}#94a3b8)".into(),
            mute_actionbar_permanent: "#f87171Chat muted #94a3b8• #f8fafc{reason}".into(),
            mute_actionbar_temporary: "#f87171Chat muted for #fbbf24{duration} #94a3b8• #f8fafc{reason}".into(),
            warn_actionbar: "#fbbf24Warning #94a3b8• #f8fafc{reason}".into(),
            warn_notice: "#fbbf24&l[!] #fbbf24You received a warning from #f8fafc{actor}#fbbf24. #94a3b8Reason: #f8fafc{reason} #94a3b8({rule_link})".into(),
            broadcast: "#f87171&l[Moderation] #f8fafc{player} #94a3b8received punishment (#f87171{kind}#94a3b8) from #f8fafc{actor}#94a3b8: #f8fafc{reason}".into(),
            actor_receipt: "#4ade80&l[Success] #f8fafc{player} #94a3b8punished (#4ade80{kind}#94a3b8): #f8fafc{reason}".into(),
            reason_by_rule: "Rule breach s.{rule}: {rule_title}".into(),
            no_account: "#fbbf24&lNETWORK ACCOUNT REQUIRED\n\n#94a3b8We don't recognize you: this username is not linked to any account.\n#94a3b8Sign in through the launcher first — it will create one and bring you back.\n\n#64748bhttps://noro.dalynkaa.dev".into(),
            no_access: "#fbbf24&lSERVER CLOSED FOR YOU\n\n#94a3b8Your account does not have access to this build.\n#94a3b8If this is an error — contact support.\n\n#64748bhttps://noro.dalynkaa.dev/support".into(),
            maintenance: "#fbbf24&lTECHNICAL MAINTENANCE\n\n#94a3b8The server is currently undergoing technical maintenance.\n\n#64748bPlease try connecting again later.".into(),
        }
    }
}

