package com.tflives.mpguard.mixin;

import com.tflives.mpguard.MpGuardState;
import net.minecraft.client.gui.screen.GameMenuScreen;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ClickableWidget;
import net.minecraft.text.Text;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Deshabilita "Abrir a LAN" en el menú de pausa (Escape dentro de una
 * partida) para cuentas offline/cracked — el bloqueo JVM de auth de
 * Mojang no cubre LAN (no necesita autenticarse contra Mojang), así que
 * sin esto una cuenta cracked igual podía compartir/entrar por LAN.
 */
@Mixin(GameMenuScreen.class)
public abstract class GameMenuScreenMixin extends Screen {

	protected GameMenuScreenMixin(Text title) {
		super(title);
	}

	@Inject(method = "init", at = @At("TAIL"))
	private void tflMpGuard$disableShareToLan(CallbackInfo ci) {
		if (!MpGuardState.isActive()) {
			return;
		}
		String target = Text.translatable("menu.shareToLan").getString();
		for (var element : this.children()) {
			if (element instanceof ClickableWidget widget
					&& target.equals(widget.getMessage().getString())) {
				widget.active = false;
			}
		}
	}
}
