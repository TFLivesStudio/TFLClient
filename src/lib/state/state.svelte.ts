import type { Settings, InstanceData, MinecraftUser } from '$lib/types/types';

export interface AppState {
	settings: Settings | null;
	instances: InstanceData[];
	selectedInstance: InstanceData | null;
	currentUser: MinecraftUser | null;
}

export const appState = $state<AppState>({
	settings: null,
	instances: [],
	selectedInstance: null,
	currentUser: null
});
