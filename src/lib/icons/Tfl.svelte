<script lang="ts">
	let {
		width = '100%',
		height = '100%',
		class: className = ''
	}: {
		width?: string | number;
		height?: string | number;
		class?: string;
	} = $props();
</script>

<svg viewBox="0 0 100 100" {width} {height} class={className} xmlns="http://www.w3.org/2000/svg">
	<path
		d="M 50 4 L 92 27 V 73 L 50 96 L 8 73 V 27 Z M 50 16 L 19 33 V 67 L 50 84 L 81 67 V 33 Z"
		fill="currentColor"
		fill-rule="evenodd"
	/>
	<rect x="42" y="38" width="16" height="16" rx="3" fill="currentColor" />
</svg>
