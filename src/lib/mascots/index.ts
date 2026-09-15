export type MascotId = 'fox' | 'bear' | 'cat' | 'owl' | 'panda' | 'frog' | 'bee' | 'penguin';

export interface MascotDef {
	id: MascotId;
	label: string;
	bg: string;
	primary: string;
	secondary: string;
}

// Paleta con dos colores de "IP" + un fondo sólido por mascota, siguiendo
// la filosofía del skill de referencia (ip-as-logo): tres colores
// semánticos nada más, fondo levemente desaturado.
export const MASCOTS: MascotDef[] = [
	{ id: 'fox', label: 'Zorro', bg: '#3a2418', primary: '#ff8c42', secondary: '#fff4e6' },
	{ id: 'bear', label: 'Oso', bg: '#2b1f16', primary: '#a9764f', secondary: '#5c4530' },
	{ id: 'cat', label: 'Gato', bg: '#241f2e', primary: '#8b7fd6', secondary: '#ffb6c1' },
	{ id: 'owl', label: 'Búho', bg: '#1f2a24', primary: '#6b8f71', secondary: '#e8a13c' },
	{ id: 'panda', label: 'Panda', bg: '#1c2320', primary: '#7fbf7f', secondary: '#1a1a1a' },
	{ id: 'frog', label: 'Rana', bg: '#142218', primary: '#5cb85c', secondary: '#e8f5e0' },
	{ id: 'bee', label: 'Abeja', bg: '#241f10', primary: '#1a1a1a', secondary: '#ffd23f' },
	{ id: 'penguin', label: 'Pingüino', bg: '#101a24', primary: '#2c3e50', secondary: '#ffb703' }
];

/** Elige una mascota "al azar" de forma determinística a partir del UUID
 *  de la cuenta — así la misma cuenta cracked siempre arranca con la
 *  misma mascota entre reinicios, sin tener que persistir nada hasta que
 *  el usuario la cambie explícitamente (eso sí se guarda, ver
 *  getMascotFor/setMascotFor). */
export function pickMascotForSeed(seed: string): MascotId {
	let hash = 0;
	for (let i = 0; i < seed.length; i++) {
		hash = (hash * 31 + seed.charCodeAt(i)) >>> 0;
	}
	return MASCOTS[hash % MASCOTS.length].id;
}

function storageKey(uuid: string) {
	return `tfl-mascot-${uuid}`;
}

export function getMascotFor(uuid: string): MascotId {
	try {
		const stored = localStorage.getItem(storageKey(uuid));
		if (stored && MASCOTS.some((m) => m.id === stored)) return stored as MascotId;
	} catch {
		// localStorage inaccesible — cae al determinístico
	}
	return pickMascotForSeed(uuid);
}

export function setMascotFor(uuid: string, id: MascotId) {
	try {
		localStorage.setItem(storageKey(uuid), id);
	} catch {
		// no-op si localStorage no está disponible
	}
}
