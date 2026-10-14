// Barrel de los componentes compartidos de la UI — Entrega A (Visual
// Foundation). Mismo patrón que src/lib/api/index.ts: importar todo desde
// '$lib/components/ui' en vez de rutas de archivo individuales.
export { default as Button } from './Button.svelte';
export { default as IconButton } from './IconButton.svelte';
export { default as Input } from './Input.svelte';
export { default as Select } from './Select.svelte';
export { default as Surface } from './Surface.svelte';
export { default as Badge } from './Badge.svelte';
export { default as Tooltip } from './Tooltip.svelte';
export { default as ProgressBar } from './ProgressBar.svelte';
export { default as Spinner } from './Spinner.svelte';
export { default as Skeleton } from './Skeleton.svelte';
export { default as EmptyState } from './EmptyState.svelte';
export { default as ErrorState } from './ErrorState.svelte';
export { default as Dialog } from './Dialog.svelte';
export { default as Popover } from './Popover.svelte';

// Componentes que ya existían antes de esta entrega — reexportados acá
// también para que el barrel sirva de punto de entrada único a toda
// src/lib/components/ui/, sin tocar esos archivos.
export { default as ConfirmDialog } from './ConfirmDialog.svelte';
export { default as ContextMenu } from './ContextMenu.svelte';
export { default as InfoTooltip } from './InfoTooltip.svelte';
export { default as Mascot } from './Mascot.svelte';
