package com.tflives.mpguard.mixin;

import com.tflives.mpguard.MpGuardState;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.client.gui.widget.ClickableWidget;
import net.minecraft.text.Text;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Deshabilita el botón "Multijugador" de la pantalla de título cuando el
 * launcher marcó la cuenta activa como offline/cracked
 * (-Dtflclient.cracked=true). Busca el botón por su TEXTO traducido
 * ("menu.multiplayer") en vez de por nombre de campo/método — los nombres
 * internos obfuscados cambian de versión en versión, la clave de
 * traducción no.
 */
@Mixin(TitleScreen.class)
public abstract class TitleScreenMixin extends Screen {

	protected TitleScreenMixin(Text title) {
		super(title);
	}

	@Inject(method = "init", at = @At("TAIL"))
	private void tflMpGuard$disableMultiplayerButton(CallbackInfo ci) {
		if (!MpGuardState.isActive()) {
			return;
		}
		String target = Text.translatable("menu.multiplayer").getString();
		for (var element : this.children()) {
			if (element instanceof ClickableWidget widget
					&& target.equals(widget.getMessage().getString())) {
				widget.active = false;
			}
		}
	}
}
