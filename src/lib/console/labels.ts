import { tr } from '$lib/i18n/experience';
import type { Metric } from './model';
export function metricLabel(metric: Metric): string {
	return metric === 'storage'
		? tr('Almacenamiento', 'Storage')
		: metric === 'network'
			? tr('Red', 'Network')
			: metric === 'gpu'
				? 'GPU'
				: metric === 'disk'
					? tr('E/S de disco', 'Disk I/O')
					: metric === 'ram'
						? tr('Memoria', 'Memory')
						: 'CPU';
}
