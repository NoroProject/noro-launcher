package dev.noro.agent.core.automod;

/**
 * Нормализация текста до проверки автомодерацией:
 * обход разрядки ("п р и в е т"), транслит/гомоглифы, leet-speak (4->a, 0->o, 3->e) и очистка символов.
 */
public final class TextNormalize {

    private TextNormalize() {}

    public static String normalize(String input) {
        if (input == null || input.isEmpty()) {
            return "";
        }

        StringBuilder sb = new StringBuilder();
        String lower = input.toLowerCase(java.util.Locale.ROOT);

        for (int i = 0; i < lower.length(); i++) {
            char c = lower.charAt(i);
            char replaced = replaceHomoglyphAndLeet(c);
            if (Character.isLetterOrDigit(replaced) || replaced == '.') {
                sb.append(replaced);
            }
        }

        return sb.toString();
    }

    private static char replaceHomoglyphAndLeet(char c) {
        switch (c) {
            // Cyrillic homoglyphs
            case 'а': return 'a';
            case 'в': return 'b';
            case 'е': return 'e';
            case 'к': return 'k';
            case 'м': return 'm';
            case 'о': return 'o';
            case 'р': return 'p';
            case 'с': return 'c';
            case 'т': return 't';
            case 'у': return 'y';
            case 'х': return 'x';

            // Leet speak substitutions
            case '0': return 'o';
            case '1': return 'i';
            case '3': return 'e';
            case '4': return 'a';
            case '5': return 's';
            case '7': return 't';
            case '@': return 'a';
            case '$': return 's';

            default: return c;
        }
    }
}
