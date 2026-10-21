export type Reaction = 'idle' | 'launch' | 'success' | 'error';
export const reaction = $state<{ kind: Reaction; sequence: number }>({ kind: 'idle', sequence: 0 });
let timer: ReturnType<typeof setTimeout> | undefined;
/** Called only by real operations/events. No progress values are synthesized. */
export function react(kind: Reaction) {
	clearTimeout(timer);
	reaction.kind = kind;
	reaction.sequence++;
	if (kind !== 'idle')
		timer = setTimeout(() => (reaction.kind = 'idle'), kind === 'launch' ? 5000 : 2200);
}
export function stopReactions() {
	clearTimeout(timer);
	reaction.kind = 'idle';
}
