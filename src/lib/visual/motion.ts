import { effects } from './runtime.svelte';
/** Cancel obsolete transitions; no cloned DOM, deferred navigation or blocked pointer events. */
export function reveal(node: HTMLElement, identity: string) {
	let animations: Animation[] = [];
	function run() {
		animations.forEach((a) => a.cancel());
		animations = [];
		const policy = effects.current;
		if (!policy.motion) return;
		const regions = policy.cinematic ? Array.from(node.children).slice(0, 5) : [node];
		animations = regions.map((region, index) =>
			region.animate(
				[
					{ opacity: 0, transform: `translateY(${policy.cinematic ? 14 : 5}px)` },
					{ opacity: 1, transform: 'translateY(0)' }
				],
				{
					duration: policy.duration,
					delay: index * policy.stagger,
					easing: 'cubic-bezier(.16,1,.3,1)',
					fill: 'backwards'
				}
			)
		);
	}
	run();
	return {
		update(next: string) {
			if (next !== identity) {
				identity = next;
				run();
			}
		},
		destroy() {
			animations.forEach((a) => a.cancel());
		}
	};
}
/** One pointer handler and one pending frame for the visible hero. */
export function parallax(node: HTMLElement, enabled: boolean) {
	let frame = 0;
	const move = (e: PointerEvent) => {
		if (!enabled || frame) return;
		const rect = node.getBoundingClientRect();
		frame = requestAnimationFrame(() => {
			frame = 0;
			node.style.setProperty(
				'--pointer-x',
				`${((e.clientX - rect.left) / rect.width - 0.5) * 10}px`
			);
			node.style.setProperty(
				'--pointer-y',
				`${((e.clientY - rect.top) / rect.height - 0.5) * 6}px`
			);
		});
	};
	const reset = () => {
		cancelAnimationFrame(frame);
		frame = 0;
		node.style.setProperty('--pointer-x', '0px');
		node.style.setProperty('--pointer-y', '0px');
	};
	node.addEventListener('pointermove', move);
	node.addEventListener('pointerleave', reset);
	return {
		update(value: boolean) {
			enabled = value;
			if (!enabled) reset();
		},
		destroy() {
			reset();
			node.removeEventListener('pointermove', move);
			node.removeEventListener('pointerleave', reset);
		}
	};
}
