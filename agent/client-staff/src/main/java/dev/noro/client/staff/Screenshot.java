package dev.noro.client.staff;

import com.mojang.blaze3d.pipeline.RenderTarget;
import com.mojang.blaze3d.platform.NativeImage;
import com.mojang.blaze3d.systems.RenderSystem;

import dev.noro.client.staff.CaseIntents;
import java.util.Base64;
import net.minecraft.client.Minecraft;

/**
 * Кадр экрана в дело, минуя папку {@code screenshots/}.
 *
 * <p>Killaura скриншотом не докажешь, а «стоял в чужом доме с чужим сундуком» —
 * вполне. Телепорт к цели и начало слежки снимают кадр сами: контекст в деле
 * появляется без напоминаний.
 */
public final class Screenshot {

    /** Подпись под кадром, если её выставили перед снимком. */
    private static volatile String note = "";

    private Screenshot() {}

    public static void note(String text) {
        note = text == null ? "" : text;
    }

    /** Снять и приложить к открытой карточке. Дела нет — нечего и прикладывать. */
    public static void attachToOpenCase() {
        CaseModels.View view = NoroStaff.state().open();
        if (view == null || view.brief() == null) {
            return;
        }
        attach(view.brief().id());
    }

    public static void attach(String caseId) {
        if (!NoroStaff.state().can(NoroStaff.PERM_CLAIM)) {
            return;
        }
        // Буфер кадра читается только на рендер-потоке; PNG собирается там же,
        // а base64 и отправка уже никому не мешают.
        RenderSystem.recordRenderCall(() -> {
            try {
                RenderTarget target = Minecraft.getInstance().getMainRenderTarget();
                try (NativeImage image = net.minecraft.client.Screenshot.takeScreenshot(target)) {
                    String png = Base64.getEncoder().encodeToString(image.asByteArray());
                    String caption = note;
                    note = "";
                    NoroStaff.cases().send(new CaseIntents.Attach(caseId, caption, png));
                }
            } catch (Exception e) {
                NoroStaff.LOG.warn("кадр не снялся: {}", e.toString());
            }
        });
    }
}
