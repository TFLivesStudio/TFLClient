import type { InstanceData, SelectionIdentity } from './types/types';

/** Provenance comes from the installed lock, independently of the loader or instance name. */
export function isSelectionInstance(instance: Pick<InstanceData, 'selection'>): boolean {
	return !!instance.selection?.target_id;
}

export function releaseLabel(release: string, channel: string): string {
	const preview = /^\d+\.\d+\.\d+-preview\.(\d+)$/.exec(release);
	const label = preview ? `Preview ${preview[1]}` : release;
	const channelLabel =
		({ beta: 'Beta', stable: 'Stable', dev: 'Dev' } as Record<string, string>)[channel] ?? channel;
	return `${label} · ${channelLabel}`;
}

export function instanceEdition(instance: InstanceData): string {
	return instance.selection
		? releaseLabel(instance.selection.release, instance.selection.channel)
		: (instance.server_type ?? instance.loader);
}

export function instanceDisplayName(instance: InstanceData): string {
	return isSelectionInstance(instance)
		? instance.name.replace(/^(?:TFL Selection|TFLClient)(?=\s|$)/i, 'TFL Client')
		: instance.name;
}

/** Existing catalog has no signed icon asset. Packaged brand art is the explicit fallback;
 * future signed target/release icons are resolved by the backend, never URLs from local state. */
export function selectionFallbackIcon(identity?: SelectionIdentity | null): string | null {
	return identity ? '/images/tfl.svg' : null;
}
