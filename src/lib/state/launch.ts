import type { RunningInstanceInfo } from '$lib/types/types';

/** 'same-account': otra instancia ya corriendo usa la misma cuenta activa —
 * puede fallar el multijugador (mismo jugador en dos lados) y suma carga de
 * CPU/RAM. 'different-account': cuentas distintas, solo pega en rendimiento
 * (igual no se puede jugar las dos a la vez con una persona).
 *
 * Compartida entre InstanceDetail.svelte (detalle de instancia) y
 * WelcomeState.svelte (botón "Jugar" de la home) — la decisión es la misma
 * en los dos lados, solo cambia el componente que la dibuja. */
export function multiInstanceWarningKind(
	running: RunningInstanceInfo[],
	myUuid: string | undefined
): 'same-account' | 'different-account' | null {
	if (running.length === 0) return null;
	const sameAccount = running.some((r) => r.account_uuid === myUuid);
	return sameAccount ? 'same-account' : 'different-account';
}
