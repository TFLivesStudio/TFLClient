<script lang="ts">
	import { onMount } from 'svelte';
	import type { InstanceData } from '$lib/types/types';
	import { Search, Boxes, Plus, Settings as SettingsIcon, Sparkles, CornerDownLeft } from 'lucide-svelte';

	let {
		instances,
		onSelectInstance,
		onCreate,
		onOpenSettings,
		onOpenTflSelection
	}: {
		instances: InstanceData[];
		onSelectInstance: (i: InstanceData) => void;
		onCreate: () => void;
		onOpenSettings: () => void;
		onOpenTflSelection: () => void;
	} = $props();

	interface PaletteItem {
		id: string;
		label: string;
		hint: string;
		icon: typeof Boxes;
		run: () => void;
	}

	let open = $state(false);
	let query = $state('');
	let selectedIndex = $state(0);
	let inputEl = $state<HTMLInputElement | undefined>(undefined);

	function close() {
		open = false;
	}

	const actionItems = $derived<PaletteItem[]>([
		{
			id: 'action:create',
			label: 'Crear instancia',
			hint: 'Acción',
			icon: Plus,
			run: () => {
				onCreate();
				close();
			}
		},
		{
			id: 'action:tfl-selection',
			label: 'TFL Selection',
			hint: 'Acción',
			icon: Sparkles,
			run: () => {
				onOpenTflSelection();
				close();
			}
		},
		{
			id: 'action:settings',
			label: 'Ajustes',
			hint: 'Acción',
			icon: SettingsIcon,
			run: () => {
				onOpenSettings();
				close();
			}
		}
	]);

	const items = $derived.by(() => {
		const instanceItems: PaletteItem[] = instances.map((inst) => ({
			id: `instance:${inst.uuid}`,
			label: inst.name,
			hint: `${inst.mc_version} · ${inst.loader}`,
			icon: Boxes,
			run: () => {
				onSelectInstance(inst);
				close();
			}
		}));
		const all = [...instanceItems, ...actionItems];
		const q = query.trim().toLowerCase();
		if (!q) return all;
		return all.filter((i) => i.label.toLowerCase().includes(q));
	});

	$effect(() => {
		if (selectedIndex >= items.length) selectedIndex = Math.max(0, items.length - 1);
	});

	function openPalette() {
		open = true;
		query = '';
		selectedIndex = 0;
		requestAnimationFrame(() => inputEl?.focus());
	}

	function handleGlobalKeydown(e: KeyboardEvent) {
		const isMod = e.metaKey || e.ctrlKey;
		if (isMod && e.key.toLowerCase() === 'k') {
			e.preventDefault();
			if (open) close();
			else openPalette();
		} else if (e.key === 'Escape' && open) {
			close();
		}
	}

	function handleInputKeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, items.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			items[selectedIndex]?.run();
		}
	}

	onMount(() => {
		window.addEventListener('keydown', handleGlobalKeydown);
		return () => window.removeEventListener('keydown', handleGlobalKeydown);
	});
</script>

{#if open}
	<div
		class="overlay"
		onclick={close}
		onkeydown={(e) => e.key === 'Escape' && close()}
		role="button"
		tabindex="-1"
	>
		<div
			class="palette anim-fade-in"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			tabindex="-1"
		>
			<div class="input-row">
				<Search size={15} class="search-icon" />
				<input
					bind:this={inputEl}
					bind:value={query}
					onkeydown={handleInputKeydown}
					placeholder="Buscar instancias o acciones…"
					type="text"
					autocomplete="off"
					spellcheck="false"
				/>
				<kbd>Esc</kbd>
			</div>
			<div class="items">
				{#if items.length === 0}
					<p class="empty">Sin resultados</p>
				{/if}
				{#each items as item, i (item.id)}
					<button
						type="button"
						class="item"
						class:active={i === selectedIndex}
						onmouseenter={() => (selectedIndex = i)}
						onclick={() => item.run()}
					>
						<item.icon size={15} class="item-icon" />
						<span class="item-label">{item.label}</span>
						<span class="item-hint">{item.hint}</span>
						{#if i === selectedIndex}<CornerDownLeft size={13} class="item-enter" />{/if}
					</button>
				{/each}
			</div>
		</div>
	</div>
{/if}

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: var(--bg-overlay);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 14vh;
		z-index: 300;
	}

	.palette {
		width: min(520px, calc(100vw - 32px));
		max-height: 60vh;
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	.input-row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 14px 16px;
		border-bottom: 1px solid var(--border);
		flex-shrink: 0;
	}

	.input-row :global(.search-icon) {
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.input-row input {
		flex: 1;
		border: none;
		background: transparent;
		outline: none;
		font-size: 0.88rem;
		color: var(--text-primary);
	}

	.input-row input::placeholder {
		color: var(--text-muted);
	}

	.input-row kbd {
		font-size: 0.65rem;
		font-weight: 700;
		color: var(--text-muted);
		background: var(--bg-input);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 2px 6px;
		flex-shrink: 0;
	}

	.items {
		overflow-y: auto;
		padding: 6px;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.empty {
		padding: 20px;
		text-align: center;
		font-size: 0.8rem;
		color: var(--text-muted);
	}

	.item {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 9px 10px;
		border: none;
		border-radius: var(--border-radius-sm);
		background: transparent;
		color: var(--text-primary);
		font-size: 0.82rem;
		text-align: left;
		cursor: pointer;
	}

	.item :global(.item-icon) {
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.item.active {
		background: color-mix(in srgb, var(--accent) 16%, var(--bg-input));
	}

	.item.active :global(.item-icon) {
		color: var(--accent);
	}

	.item-label {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.item-hint {
		font-size: 0.68rem;
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.item :global(.item-enter) {
		color: var(--accent);
		flex-shrink: 0;
	}
</style>
