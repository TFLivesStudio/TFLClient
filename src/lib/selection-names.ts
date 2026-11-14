/** Normalize legacy catalog labels for display, preserving signed bytes and instance IDs. */
export function clientDisplayName(name: string | undefined, minecraftVersion: string): string {
	const value = name?.trim();
	return value
		? value.replace(/^(?:TFL Selection|TFLClient)(?=\s|$)/i, 'TFL Client')
		: `TFL Client ${minecraftVersion}`;
}
