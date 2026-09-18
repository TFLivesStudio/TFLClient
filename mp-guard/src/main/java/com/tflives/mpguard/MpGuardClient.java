package com.tflives.mpguard;

import net.fabricmc.api.ClientModInitializer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class MpGuardClient implements ClientModInitializer {
	public static final Logger LOGGER = LoggerFactory.getLogger("tfl-mp-guard");

	@Override
	public void onInitializeClient() {
		if (MpGuardState.isActive()) {
			LOGGER.info("TFL MP Guard activo — cuenta offline/cracked, multijugador bloqueado.");
		} else {
			LOGGER.info("TFL MP Guard presente pero inactivo (cuenta no marcada como cracked).");
		}
	}
}
