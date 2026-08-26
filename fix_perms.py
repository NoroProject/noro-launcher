import re

with open('crates/schema/src/permissions.rs', 'r', encoding='utf-8') as f:
    content = f.read()

pattern = re.compile(r'^\s*(PERM_[A-Z0-9_]+)\s*=\s*"([^"]+)",\s*"([^"]+)",\s*"([^"]+)";', re.MULTILINE)

ru_entries = []
en_entries = []
groups = {}

def process_match(m):
    const_name = m.group(1)
    node = m.group(2)
    group_ru = m.group(3)
    title_ru = m.group(4)
    
    group_key = f"perm-group-{group_ru.lower()}"
    # translate some common groups to English keys for better looking keys
    # But it's easier to just use a hash or manual mapping.
    group_map = {
        "Панель": "panel",
        "Игроки": "players",
        "Модерация": "moderation",
        "Правила": "rules",
        "Серверы": "servers",
        "Сборки": "builds",
        "Машина": "machine",
        "Контент": "content",
        "Лаунчер": "launcher",
        "Защита": "security",
        "Поддержка": "support",
        "Система": "system",
        "Игрок": "player"
    }
    
    g_key = "perm-group-" + group_map.get(group_ru, "other")
    
    if g_key not in groups:
        groups[g_key] = group_ru
        
    title_key = f"perm-node-{const_name.lower().replace('perm_', '').replace('_', '-')}"
    
    ru_entries.append(f"{title_key} = {title_ru}")
    en_entries.append(f"{title_key} = {title_ru} (EN)") # Placeholder
    
    return f'    {const_name:<25} = "{node}", "{g_key}", "{title_key}";'

new_content = pattern.sub(process_match, content)

with open('crates/schema/src/permissions.rs', 'w', encoding='utf-8') as f:
    f.write(new_content)

ru_lines = []
for k, v in groups.items():
    ru_lines.append(f"{k} = {v}")
ru_lines.extend(ru_entries)

with open('crates/i18n/locales/ru.ftl', 'a', encoding='utf-8') as f:
    f.write("\n\n# Permission Nodes\n")
    f.write("\n".join(ru_lines) + "\n")

en_lines = []
for k, v in groups.items():
    en_lines.append(f"{k} = {v} (EN)")
en_lines.extend(en_entries)

with open('crates/i18n/locales/en.ftl', 'a', encoding='utf-8') as f:
    f.write("\n\n# Permission Nodes\n")
    f.write("\n".join(en_lines) + "\n")

print("Done")
