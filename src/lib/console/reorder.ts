import type { Metric } from './model';
import { dropMetric } from './preferences.svelte';
/** Pointer handles work with Tauri's native file-drop handler enabled on Windows.
 * HTML5 drag/drop remains available on cards in other webviews. */
export function reorderHandle(
	node: HTMLElement,
	options: Metric | { metric: Metric; drop: (from: Metric, to: Metric) => void }
) {
	let params = options;
	let id: number | null = null;
	let target: HTMLElement | null = null;
	const clear = () => {
		target?.removeAttribute('data-drop-target');
		target = null;
	};
	const down = (e: PointerEvent) => {
		if (e.button !== 0) return;
		e.preventDefault();
		e.stopPropagation();
		id = e.pointerId;
		node.setPointerCapture(id);
		node.focus();
		node.setAttribute('aria-grabbed', 'true');
	};
	const move = (e: PointerEvent) => {
		if (e.pointerId !== id) return;
		const found =
			document
				.elementFromPoint(e.clientX, e.clientY)
				?.closest<HTMLElement>('[data-console-metric]') ?? null;
		const scope = node.closest('[data-metric-list]');
		clear();
		if (found && scope?.contains(found)) {
			target = found;
			target.setAttribute('data-drop-target', '');
		}
	};
	const up = (e: PointerEvent) => {
		if (e.pointerId !== id) return;
		const to = target?.dataset.consoleMetric;
		if (e.type === 'pointerup' && (to === 'cpu' || to === 'ram' || to === 'disk'))
			(typeof params === 'string' ? dropMetric : params.drop)(
				typeof params === 'string' ? params : params.metric,
				to
			);
		if (node.hasPointerCapture(e.pointerId)) node.releasePointerCapture(e.pointerId);
		id = null;
		node.removeAttribute('aria-grabbed');
		clear();
	};
	node.addEventListener('pointerdown', down);
	node.addEventListener('pointermove', move);
	node.addEventListener('pointerup', up);
	node.addEventListener('pointercancel', up);
	return {
		update(options: typeof params) {
			params = options;
		},
		destroy() {
			clear();
			node.removeEventListener('pointerdown', down);
			node.removeEventListener('pointermove', move);
			node.removeEventListener('pointerup', up);
			node.removeEventListener('pointercancel', up);
		}
	};
}
