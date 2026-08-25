package dev.noro.client.staff.ui;

import dev.noro.client.staff.NoroStaff;

import dev.noro.client.staff.CaseModels;
import dev.noro.client.ui.Surface;
import dev.noro.client.ui.Theme;
import dev.noro.client.staff.CaseIntents;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.network.chat.Component;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.phys.EntityHitResult;
import net.minecraft.world.phys.HitResult;

/**
 * Досье при наведении: ник, роль, сколько дел подтвердилось, активные санкции.
 *
 * <p>Тот же вопрос «он новичок или у него третий бан за то же», который сейчас
 * требует ухода на сайт посреди разбора.
 */
public final class DossierHud {

    /** Кого спрашивали последним: без этого запрос уходил бы каждый кадр. */
    private static String asked = "";

    private DossierHud() {}

    public static void render(GuiGraphics g, int width, int height) {
        Minecraft mc = Minecraft.getInstance();
        if (mc.player == null || !NoroStaff.state().can(NoroStaff.PERM_VIEW)) {
            return;
        }
        String name = aimedAt(mc);
        if (name == null) {
            asked = "";
            return;
        }
        if (!name.equals(asked)) {
            asked = name;
            NoroStaff.cases().send(new CaseIntents.Lookup(name));
        }

        CaseModels.Dossier d = NoroStaff.state().dossier();
        if (d == null || !name.equals(d.username())) {
            return;
        }
        card(g, d, width, height);
    }

    private static String aimedAt(Minecraft mc) {
        HitResult hit = mc.hitResult;
        if (!(hit instanceof EntityHitResult entityHit)) {
            return null;
        }
        Entity entity = entityHit.getEntity();
        return entity instanceof Player ? entity.getName().getString() : null;
    }

    private static void card(GuiGraphics g, CaseModels.Dossier d, int width, int height) {
        Minecraft mc = Minecraft.getInstance();
        int w = 44 * Theme.GRID;
        int h = 20 * Theme.GRID;
        int x = width / 2 + 6 * Theme.GRID;
        int y = height / 2 - h / 2;
        Surface.card(g, x, y, w, h);

        g.drawString(mc.font, d.username(), x + 2 * Theme.GRID, y + 2 * Theme.GRID,
                Theme.TEXT);
        String roles = d.roles() == null || d.roles().isEmpty() ? "—" : String.join(", ", d.roles());
        g.drawString(mc.font, roles, x + 2 * Theme.GRID, y + 6 * Theme.GRID, Theme.TEXT_MUTED);
        g.drawString(
                mc.font,
                Component.translatable("noro.cases.dossier.cases", d.cases_confirmed(),
                        d.cases_total()),
                x + 2 * Theme.GRID,
                y + 10 * Theme.GRID,
                d.cases_confirmed() > 0 ? Theme.ERROR : Theme.TEXT_MUTED);
        int active = d.active_punishments() == null ? 0 : d.active_punishments().size();
        g.drawString(
                mc.font,
                Component.translatable("noro.cases.dossier.active", active),
                x + 2 * Theme.GRID,
                y + 14 * Theme.GRID,
                active > 0 ? Theme.ERROR : Theme.TEXT_MUTED);
        if (d.first_seen() != null && d.first_seen().length() >= 10) {
            g.drawString(
                    mc.font,
                    Component.translatable("noro.cases.dossier.since",
                            d.first_seen().substring(0, 10)),
                    x + 2 * Theme.GRID,
                    y + h - 5 * Theme.GRID,
                    Theme.TEXT_MUTED);
        }
    }
}
