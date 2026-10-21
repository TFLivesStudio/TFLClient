import { tick } from 'svelte';
import { effects } from './runtime.svelte';
let pending: ViewTransition | undefined;
/** Navigation updates immediately inside the snapshot callback; operations never await effects. */
export function transitionToInstance(id: string, change: () => void) {
	if (typeof document === 'undefined' || !document.startViewTransition || !effects.current.motion) {
		change();
		return;
	}
	pending?.skipTransition();
	const selector = `.main-content [data-instance-artwork="${CSS.escape(id)}"]`;
	const source = document.querySelector<HTMLElement>(selector);
	if (!source) {
		change();
		return;
	}
	source.style.viewTransitionName = 'instance-artwork';
	let destination: HTMLElement | null = null;
	const result = document.startViewTransition(async () => {
		change();
		await tick();
		source.style.viewTransitionName = '';
		destination = document.querySelector<HTMLElement>(selector);
		if (destination) destination.style.viewTransitionName = 'instance-artwork';
	});
	pending = result;
	void result.ready.catch(() => {
		/* Unsupported snapshots fall back to the existing reveal. */
	});
	void result.finished.finally(() => {
		source.style.viewTransitionName = '';
		if (destination) destination.style.viewTransitionName = '';
		if (pending === result) pending = undefined;
	});
}
