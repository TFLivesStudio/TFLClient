<script lang="ts">
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { getInstanceIconPath } from '$lib/api';
	import { iconVersions } from '$lib/state/instanceState.svelte';
	import { Box, Server } from 'lucide-svelte';
	import type { InstanceData } from '$lib/types/types';
	let { instance }: { instance: InstanceData } = $props();
	let url = $state<string | null>(null);
	$effect(() => {
		const name = instance.name,
			version = iconVersions[name] ?? 0;
		let active = true;
		getInstanceIconPath(name)
			.then((path) => {
				if (active) url = path ? `${convertFileSrc(path)}?v=${version}` : null;
			})
			.catch(() => {
				if (active) url = null;
			});
		return () => {
			active = false;
		};
	});
</script>

<span class="mark" data-instance-artwork={instance.uuid}
	>{#if url}<img src={url} alt="" loading="lazy" />{:else if instance.server_type}<Server
			size={24}
		/>{:else}<Box size={26} />{/if}</span
>

<style>
	.mark {
		display: grid;
		place-items: center;
		width: 52px;
		height: 52px;
		flex-shrink: 0;
		border-radius: var(--border-radius);
		color: var(--accent);
		background: linear-gradient(
			140deg,
			color-mix(in srgb, var(--accent) 19%, var(--bg-card)),
			var(--bg-card)
		);
		border: 1px solid var(--border);
		overflow: hidden;
	}
	.mark img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
</style>
