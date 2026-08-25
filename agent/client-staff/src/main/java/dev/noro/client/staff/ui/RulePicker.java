package dev.noro.client.staff.ui;

import dev.noro.client.NoroCore;
import dev.noro.client.rules.RuleBook;
import dev.noro.client.ui.RowList;
import dev.noro.client.ui.TextField;
import dev.noro.client.ui.Theme;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;

/**
 * Колонка выбора правила: поиск и список свода.
 *
 * <p>Список, а не свободный ввод: код правила попадает в наказание ровно таким,
 * какой записан в своде, — по нему потом разбирают жалобу на самого модератора.
 *
 * <p>Отдельно от экрана по той же причине, что и на сайте: выбор правила —
 * законченный кусок, который живёт своей жизнью (поиск, прокрутка, выделение), и
 * в одном файле с формой наказания его не найти.
 */
final class RulePicker {

    private final RowList<RuleBook.Rule> list = new RowList<>(RulePicker::row);

    private TextField search;
    private int x;
    private int y;
    private int w;
    private int h;

    /** Поле поиска: создаёт его выборщик, а на экран добавляет экран. */
    TextField search(Font font, int x, int y, int w) {
        search = TextField.of(font, x, y, w,
                Component.translatable("noro.cases.punish.rule"),
                Component.translatable("noro.cases.punish.rule.search"));
        search.setResponder(text -> list.items(NoroCore.rules().search(text)));
        return search;
    }

    /** Куда лечь списку: считает экран, он один знает про обе колонки. */
    void bounds(int x, int y, int w, int h) {
        this.x = x;
        this.y = y;
        this.w = w;
        this.h = h;
    }

    void render(GuiGraphics g, int mouseX, int mouseY) {
        if (!NoroCore.rules().loaded()) {
            g.drawString(Theme.font(), Component.translatable("noro.cases.punish.rule.loading"),
                    x, y, Theme.TEXT_MUTED, false);
            return;
        }
        if (list.empty()) {
            list.items(NoroCore.rules().search(search == null ? "" : search.getValue()));
        }
        list.render(g, x, y, w, h, mouseX, mouseY);
    }

    boolean click(double mouseX, double mouseY) {
        return list.click(mouseX, mouseY);
    }

    void scroll(double delta) {
        list.scroll(delta);
    }

    RuleBook.Rule selected() {
        return list.selected();
    }

    private static void row(RowList.Row row, RuleBook.Rule rule) {
        Font font = Theme.font();
        row.g().drawString(font, rule.code(), row.x(), row.y(), Theme.CTA, false);
        // Название начинается за самым длинным кодом, а не за своим: иначе
        // столбец названий гуляет от строки к строке.
        int offset = Math.max(font.width(rule.code()) + 2 * Theme.GRID, 8 * Theme.GRID);
        row.g().drawString(font, Theme.clip(Component.literal(rule.title()), row.w() - offset),
                row.x() + offset, row.y(), row.active() ? Theme.TEXT : Theme.TEXT_MUTED, false);
    }
}
