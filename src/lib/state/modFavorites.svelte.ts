export interface FavoriteMod {
	projectId: string;
	title: string;
	iconUrl: string | null;
}

const STORAGE_KEY = 'tfl-favorite-mods';

function load(): FavoriteMod[] {
	if (typeof localStorage === 'undefined') return [];
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		return raw ? JSON.parse(raw) : [];
	} catch {
		return [];
	}
}

// Colección personal de mods favoritos, cross-instancia — un mod se marca
// favorito una vez y desde ahí se puede instalar rápido en cualquier
// instancia sin tener que volver a buscarlo. Solo localStorage, no hace
// falta ida y vuelta a Rust para algo así de simple.
export const favoriteMods = $state<FavoriteMod[]>(load());

function persist() {
	if (typeof localStorage === 'undefined') return;
	localStorage.setItem(STORAGE_KEY, JSON.stringify(favoriteMods));
}

export function isFavoriteMod(projectId: string): boolean {
	return favoriteMods.some((m) => m.projectId === projectId);
}

export function toggleFavoriteMod(mod: FavoriteMod) {
	const idx = favoriteMods.findIndex((m) => m.projectId === mod.projectId);
	if (idx >= 0) {
		favoriteMods.splice(idx, 1);
	} else {
		favoriteMods.push(mod);
	}
	persist();
}
