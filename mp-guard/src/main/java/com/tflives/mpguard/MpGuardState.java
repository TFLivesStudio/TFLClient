package com.tflives.mpguard;

/**
 * Único punto de verdad de si el guard debe estar activo. El launcher
 * (Rust) es quien decide esto, no el mod — solo pasa
 * {@code -Dtflclient.cracked=true} en la línea de comandos de Java cuando
 * la cuenta activa es offline/cracked. Si el jar terminó instalado en una
 * instancia por error, o alguien lo copia a mano a otro launcher, el
 * default es "no hacer nada" — nunca bloquea sin que se lo pidan
 * explícitamente.
 */
public final class MpGuardState {
	private static final boolean ACTIVE =
			Boolean.getBoolean("tflclient.cracked");

	private MpGuardState() {}

	public static boolean isActive() {
		return ACTIVE;
	}
}
