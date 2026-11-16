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
