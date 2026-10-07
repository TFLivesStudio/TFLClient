import { Blocks, FileOutput, FolderOpen, Play, Rocket } from 'lucide-svelte';
import { createInstanceShortcut, exportInstanceAsMrpack, openInstanceFolder } from '$lib/api';
import type { ContextMenuItem } from '$lib/components/ui/ContextMenu.svelte';
import { t } from '$lib/i18n/index.svelte';
import { openInstanceTab } from '$lib/state/instanceState.svelte';
import type { InstanceData } from '$lib/types/types';

// Menú contextual (click derecho en una instancia de la barra lateral) —
// pedido del cliente: accesos rápidos sin tener que abrir la instancia y
// buscar el botón a mano.
export function buildInstanceContextMenuItems(instance: InstanceData): ContextMenuItem[] {
	const items: ContextMenuItem[] = [
		{
			id: 'open',
			label: t('sidebar.contextOpen'),
			icon: Play,
			run: () => openInstanceTab(instance, 'details')
		}
	];
	if (!instance.server_type) {
		items.push({
			id: 'mods',
			label: t('instanceDetail.tabs.mods'),
			icon: Blocks,
			run: () => openInstanceTab(instance, 'mods')
		});
		items.push({
			id: 'shortcut',
			label: t('instanceDetail.createShortcut'),
			icon: Rocket,
			run: () => createInstanceShortcut(instance.name)
		});
		items.push({
			id: 'export',
			label: t('instanceDetail.exportMrpack'),
			icon: FileOutput,
			run: () => exportInstanceAsMrpack(instance.name)
		});
	}
	items.push({
		id: 'folder',
		label: t('instanceDetail.folder'),
		icon: FolderOpen,
		run: () => openInstanceFolder(instance.name)
	});
	return items;
}
