package dev.noro.client.staff.ui;

import dev.noro.client.NoroCore;
import dev.noro.client.rules.RuleBook;
import dev.noro.client.ui.Chip;
import dev.noro.client.ui.Select;
import dev.noro.client.ui.TextArea;
import dev.noro.client.ui.TextField;
import dev.noro.client.ui.Theme;
import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.network.chat.Component;

/** Правая колонка формы наказания: чипы, вид, срок и причина. */
final class PunishFields {

    private static final int LABEL = 3 * Theme.GRID;
    private static final int ROW = 5 * Theme.GRID;
    private static final int GAP = 2 * Theme.GRID;

    /** Быстрые сроки: ими закрывается девять случаев из десяти. */
    private static final List<String> QUICK = List.of("15m", "1h", "12h", "1d", "7d", "30d", "");

    /** Виды, пока правило не выбрано: свод сузит список, когда выберут. */
    private static final List<String> ALL_KINDS = List.of("warn", "mute", "ban");

    private final PunishForm form;
    private final Runnable rebuild;

    private TextField duration;
    private TextArea reason;
    private int x;
    private int chipsY;
    private int kindY;
    private int reasonY;

    PunishFields(PunishForm form, Runnable rebuild) {
        this.form = form;
        this.rebuild = rebuild;
    }

    /** Собрать колонку сверху вниз: каждый блок знает только свою высоту. */
    void layout(Consumer<AbstractWidget> sink, int x, int y, int w, int bottom) {
        this.x = x;
        chipsY = y;
        y = chips(sink, x, y + LABEL, w) + GAP;

        kindY = y;
        y += LABEL;
        Select<String> kind =
                Select.of(x, y, w, form.kinds(ALL_KINDS), PunishForm::kindLabel, form::kind);
        kind.value(form.kind());
        sink.accept(kind);
        y += ROW + GAP;

        duration = field(sink, x, y, w, "noro.cases.punish.span.forever", form.duration());
        y += ROW + GAP;

        reasonY = y;
        // Причина — многострочная и во всё оставшееся место: в неё вписывают
        // цитату из чата, а в одну строку она не влезает и уезжает за край.
        reason = TextArea.of(Theme.font(), x, y + LABEL, w,
                Math.max(3 * ROW, bottom - (y + LABEL)),
                Component.translatable("noro.cases.punish.reason"),
                Component.translatable("noro.cases.punish.reason"));
        reason.setValue(form.reason());
        sink.accept(reason);
    }

    private TextField field(Consumer<AbstractWidget> sink, int x, int y, int w, String hint, String value) {
        Component placeholder = Component.translatable(hint);
        TextField field = TextField.of(Theme.font(), x, y, w, placeholder, placeholder);
        field.setValue(value);
        sink.accept(field);
        return field;
    }

    /**
     * Ряд чипов над видом наказания, с переносом.
     *
     * <p>Правило выбрано — его вилки: чип ставит и вид, и нижнюю границу срока
     * одним нажатием, ради этого свод и заполняли. Не выбрано — быстрые сроки:
     * место одно и то же, и пустой строки на экране не остаётся.
     *
     * <p>Больше двух рядов не бывает: третий пришлось бы отнять у причины или у
     * кнопок, без которых форма не отправляется вовсе. Не влезшая вилка всё
     * равно набирается полями ниже, а рамки проверяет мастер.
     *
     * @return низ занятого чипами места
     */
    private int chips(Consumer<AbstractWidget> sink, int x, int y, int w) {
        RuleBook.Rule picked = form.rule();
        int cursor = x;
        int line = y;
        for (Chip chip : picked == null ? quick() : sanctions(picked)) {
            if (cursor > x && cursor + chip.getWidth() > x + w) {
                if (line >= y + ROW) {
                    break;
                }
                cursor = x;
                line += ROW;
            }
            chip.setPosition(cursor, line);
            cursor += chip.getWidth() + Theme.GRID;
            sink.accept(chip);
        }
        return line + ROW;
    }

    private List<Chip> sanctions(RuleBook.Rule picked) {
        List<Chip> out = new ArrayList<>();
        for (RuleBook.Sanction sanction : NoroCore.rules().sanctionsOf(picked.id())) {
            out.add(Chip.of(0, 0, form.sanctionLabel(sanction), form.picked(sanction), () -> {
                form.pick(sanction);
                rebuild.run();
            }));
        }
        return out;
    }

    private List<Chip> quick() {
        List<Chip> out = new ArrayList<>();
        for (String span : QUICK) {
            Component label = span.isEmpty()
                    ? Component.translatable("noro.cases.punish.span.forever")
                    : Component.literal(span);
            out.add(Chip.of(0, 0, label, form.duration().equals(span), () -> {
                form.duration(span);
                duration.setValue(span);
            }));
        }
        return out;
    }

    /** Подписи блоков: рисуются там же, где виджеты расставлены. */
    void labels(GuiGraphics g) {
        Theme.label(g, Component.translatable(form.rule() == null
                ? "noro.cases.punish.span.quick"
                : "noro.cases.punish.allowed"), x, chipsY);
        Theme.label(g, Component.translatable("noro.cases.punish.kind"), x, kindY);
        Theme.label(g, Component.translatable("noro.cases.punish.reason"), x, reasonY);
    }

    String duration() {
        return duration.getValue();
    }

    String reason() {
        return reason.getValue();
    }

    void reason(String text) {
        reason.setValue(text);
    }
}
