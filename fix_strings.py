import json

for lang_file, text in [('ru', '<#f87171>Ваниш доступен только во время разбора жалобы.'), 
                        ('en', '<#f87171>Vanish is only available during an active case.')]:
    path = f'agent/core/src/main/resources/lang/{lang_file}.json'
    with open(path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    data["vanish_case_only"] = text
    with open(path, 'w', encoding='utf-8') as f:
        json.dump(data, f, ensure_ascii=False, indent=2)

print("Done")
