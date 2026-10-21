import type { Sample } from './model';
export const DEMO_END = 180000;
export const DEMO_HISTORY: Sample[] = Array.from({ length: 181 }, (_, i) => ({
	time: i * 1000,
	cpu: 180 + Math.sin(i / 8) * 125 + Math.cos(i / 3) * 35,
	total: (180 + Math.sin(i / 8) * 125 + Math.cos(i / 3) * 35) / 8,
	ram: 2100 + Math.sin(i / 16) * 140 + i * 2,
	disk: Math.max(0, Math.sin(i / 11) * 4)
}));
export const DEMO_LINES = [
	{ severity: 'info', line: '[12:04:01 INFO] Starting Minecraft…' },
	{ severity: 'debug', line: '[12:04:02 DEBUG] Runtime ready' },
	{ severity: 'warn', line: '[12:04:03 WARN] Resource pack uses an older format' },
	{ severity: 'error', line: '[12:04:04 ERROR] Example connection failure' },
	{ severity: 'info', line: '[12:04:05 INFO] World loaded' }
] as const;
