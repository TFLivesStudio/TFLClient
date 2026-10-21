import { runtime } from './runtime.svelte';
/** Shared headless modal behavior. Topmost modal owns Escape and Tab; styles are independent. */
const stack: HTMLElement[] = [];
const selector =
	'button:not(:disabled),a[href],input:not(:disabled),select:not(:disabled),textarea:not(:disabled),[tabindex]:not([tabindex="-1"])';
export function modalFocus(node: HTMLElement, close: (() => void) | undefined) {
	const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
	const restored: Array<[HTMLElement, boolean]> = [];
	let parent: HTMLElement | null = node;
	while (parent && parent !== document.body) {
		if (parent.parentElement)
			for (const sibling of parent.parentElement.children)
				if (
					sibling !== parent &&
					sibling instanceof HTMLElement &&
					!['SCRIPT', 'STYLE'].includes(sibling.tagName)
				) {
					restored.push([sibling, sibling.inert]);
					sibling.inert = true;
				}
		parent = parent.parentElement;
	}
	stack.push(node);
	runtime.modalDepth += 1;
	node.tabIndex = -1;
	if (!node.getAttribute('aria-label') && !node.getAttribute('aria-labelledby'))
		node.setAttribute('aria-label', node.querySelector('h1,h2,h3')?.textContent ?? 'TFL Client');
	const focusables = () =>
		Array.from(node.querySelectorAll<HTMLElement>(selector)).filter(
			(el) => !el.closest('[inert]') && el.getClientRects().length > 0
		);
	queueMicrotask(() => {
		if (stack.at(-1) === node) (focusables()[0] ?? node).focus();
	});
	const key = (e: KeyboardEvent) => {
		if (stack.at(-1) !== node) return;
		if (e.key === 'Escape') {
			e.preventDefault();
			e.stopImmediatePropagation();
			close?.();
			return;
		}
		if (e.key !== 'Tab') return;
		const list = focusables(),
			index = list.indexOf(document.activeElement as HTMLElement);
		if (!list.length) {
			e.preventDefault();
			node.focus();
			return;
		}
		if (e.shiftKey && index <= 0) {
			e.preventDefault();
			list.at(-1)?.focus();
		} else if (!e.shiftKey && (index === list.length - 1 || index === -1)) {
			e.preventDefault();
			list[0].focus();
		}
	};
	window.addEventListener('keydown', key, true);
	return {
		update(next: typeof close) {
			close = next;
		},
		destroy() {
			window.removeEventListener('keydown', key, true);
			stack.splice(stack.indexOf(node), 1);
			runtime.modalDepth = Math.max(0, runtime.modalDepth - 1);
			for (const [el, inert] of restored) el.inert = inert;
			queueMicrotask(() => {
				if (previous?.isConnected && !previous.closest('[inert]')) previous.focus();
			});
		}
	};
}
