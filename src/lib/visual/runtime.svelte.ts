import { appearance } from '$lib/state/appearanceState.svelte';
import { resolveCapabilities } from './policy';
export const runtime = $state({
	visible: true,
	focused: true,
	systemReducedMotion: false,
	overlay: false,
	modalDepth: 0
});
export const effects = {
	get current() {
		return resolveCapabilities({
			quality: appearance.quality,
			reducedMotion: appearance.reduceMotion,
			systemReducedMotion: runtime.systemReducedMotion,
			noBlur: appearance.noBlur,
			noInfiniteFx: appearance.noInfiniteFx,
			active: runtime.visible && runtime.focused
		});
	}
};
export function startVisualRuntime() {
	const media = matchMedia('(prefers-reduced-motion: reduce)');
	const sync = () => {
		runtime.visible = !document.hidden;
		runtime.focused = document.hasFocus();
		runtime.systemReducedMotion = media.matches;
		document.documentElement.toggleAttribute('data-system-reduce-motion', media.matches);
		document.documentElement.toggleAttribute(
			'data-visual-paused',
			!runtime.visible || !runtime.focused
		);
	};
	sync();
	media.addEventListener('change', sync);
	document.addEventListener('visibilitychange', sync);
	window.addEventListener('focus', sync);
	window.addEventListener('blur', sync);
	return () => {
		media.removeEventListener('change', sync);
		document.removeEventListener('visibilitychange', sync);
		window.removeEventListener('focus', sync);
		window.removeEventListener('blur', sync);
	};
}
