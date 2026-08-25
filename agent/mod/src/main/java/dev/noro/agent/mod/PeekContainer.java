package dev.noro.agent.mod;

import net.minecraft.world.Container;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;

/**
 * Чужой инвентарь как контейнер сундука: то, что модератор видит по invsee.
 *
 * <p>Обёртка, а не сам {@code Inventory}, по двум причинам. Первая — размер:
 * инвентарь игрока это 41 слот (36 своих, броня и левая рука), а меню сундука
 * принимает только кратное девяти. Показываем ровно 36 основных: броня и щит
 * видны снимком на вкладке дела, а слоты-пустышки в окне ничем не отличались бы
 * от настоящих — положенное туда пропало бы при закрытии.
 *
 * <p>Вторая — {@code stillValid}: ванильный инвентарь считает окно своим, только
 * пока смотрящий стоит рядом с хозяином. Разбор ведут и с другого конца карты, и
 * там окно закрывалось бы сразу после открытия, будто кнопка не работает.
 *
 * <p>Предметы настоящие: изъять улику или вернуть украденное можно прямо здесь, и
 * правку сразу видит сам игрок.
 */
final class PeekContainer implements Container {

    /** Основная часть инвентаря: девять слотов пояса и три ряда рюкзака. */
    private static final int SIZE = 36;

    private final Container inventory;

    PeekContainer(Container inventory) {
        this.inventory = inventory;
    }

    @Override
    public int getContainerSize() {
        return SIZE;
    }

    @Override
    public boolean isEmpty() {
        for (int slot = 0; slot < SIZE; slot++) {
            if (!inventory.getItem(slot).isEmpty()) {
                return false;
            }
        }
        return true;
    }

    @Override
    public ItemStack getItem(int slot) {
        return inventory.getItem(slot);
    }

    @Override
    public ItemStack removeItem(int slot, int count) {
        return inventory.removeItem(slot, count);
    }

    @Override
    public ItemStack removeItemNoUpdate(int slot) {
        return inventory.removeItemNoUpdate(slot);
    }

    @Override
    public void setItem(int slot, ItemStack stack) {
        inventory.setItem(slot, stack);
    }

    @Override
    public void setChanged() {
        inventory.setChanged();
    }

    @Override
    public boolean stillValid(Player player) {
        return true;
    }

    @Override
    public void clearContent() {
        for (int slot = 0; slot < SIZE; slot++) {
            inventory.setItem(slot, ItemStack.EMPTY);
        }
    }
}
