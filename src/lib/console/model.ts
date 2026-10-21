export type Metric = 'cpu' | 'ram' | 'disk';
export type Severity = 'info' | 'warn' | 'error' | 'debug';
export interface ConsolePreferences {
	version: 1;
	order: Metric[];
	visible: Record<Metric, boolean>;
	colors: Record<Metric | Severity, string>;
	dock: 'top' | 'bottom' | 'side';
	columns: 1 | 2 | 3;
	seconds: 30 | 60 | 180;
	cpuMode: 'cores' | 'total';
	/** Estilo de las tarjetas de métricas: líneas históricas (default) o anillos. */
	chartStyle: 'lines' | 'gauge';
	metricStyles: Record<Metric, 'lines' | 'gauge'>;
	metricSeconds: Record<Metric, 30 | 60 | 180>;
	fontSize: number;
	wrap: boolean;
	follow: boolean;
}
export const DEFAULT_CONSOLE: ConsolePreferences = {
	version: 1,
	order: ['cpu', 'ram', 'disk'],
	visible: { cpu: true, ram: true, disk: false },
	colors: {
		cpu: '#67e8a3',
		ram: '#93c5fd',
		disk: '#c4b5fd',
		info: '#d4d4d8',
		warn: '#facc15',
		error: '#fda4af',
		debug: '#a1a1aa'
	},
	dock: 'top',
	columns: 2,
	seconds: 60,
	cpuMode: 'cores',
	chartStyle: 'lines',
	metricStyles: { cpu: 'lines', ram: 'lines', disk: 'lines' },
	metricSeconds: { cpu: 60, ram: 60, disk: 60 },
	fontSize: 12,
	wrap: true,
	follow: true
};
export const CONSOLE_KEY = 'tfl-console-v1';
const metrics: Metric[] = ['cpu', 'ram', 'disk'];
export function parseConsole(input: unknown): ConsolePreferences {
	const d = structuredClone(DEFAULT_CONSOLE);
	if (!input || typeof input !== 'object') return d;
	const p = input as Partial<ConsolePreferences>;
	if (p.version !== undefined && p.version !== 1) return d;
	if (Array.isArray(p.order))
		d.order = [...new Set(p.order.filter((m) => metrics.includes(m))), ...metrics].filter(
			(m, i, a) => a.indexOf(m) === i
		);
	for (const m of metrics) if (typeof p.visible?.[m] === 'boolean') d.visible[m] = p.visible[m];
	for (const k of Object.keys(d.colors) as (Metric | Severity)[])
		if (typeof p.colors?.[k] === 'string' && /^#[0-9a-f]{6}$/i.test(p.colors[k]))
			d.colors[k] = p.colors[k];
	if (['top', 'bottom', 'side'].includes(p.dock ?? '')) d.dock = p.dock!;
	if ([1, 2, 3].includes(p.columns ?? 0)) d.columns = p.columns!;
	if ([30, 60, 180].includes(p.seconds ?? 0)) d.seconds = p.seconds!;
	if (p.cpuMode === 'cores' || p.cpuMode === 'total') d.cpuMode = p.cpuMode;
	if (p.chartStyle === 'lines' || p.chartStyle === 'gauge') d.chartStyle = p.chartStyle;
	for (const metric of metrics) {
		d.metricStyles[metric] = ['lines', 'gauge'].includes(p.metricStyles?.[metric] ?? '')
			? p.metricStyles![metric]
			: d.chartStyle;
		d.metricSeconds[metric] = [30, 60, 180].includes(p.metricSeconds?.[metric] ?? 0)
			? p.metricSeconds![metric]
			: d.seconds;
	}
	if (typeof p.fontSize === 'number' && Number.isFinite(p.fontSize))
		d.fontSize = Math.round(Math.max(10, Math.min(20, p.fontSize)));
	if (typeof p.wrap === 'boolean') d.wrap = p.wrap;
	if (typeof p.follow === 'boolean') d.follow = p.follow;
	return d;
}
export function severity(line: string, stream: string): Severity {
	// Match level tokens, not words inside a message (e.g. "0 errors").
	// ANSI SGR escape sequences from Java terminals.
	// eslint-disable-next-line no-control-regex
	const clean = line.replace(/\x1b\[[0-9;]*m/g, '');
	const level = clean
		.match(/(?:\[|\/|^|\s)(WARN(?:ING)?|ERROR|FATAL|INFO|DEBUG|TRACE)(?=\]|:|\s|$)/i)?.[1]
		?.toUpperCase();
	if (level === 'WARN' || level === 'WARNING') return 'warn';
	if (level === 'ERROR' || level === 'FATAL') return 'error';
	if (level === 'DEBUG' || level === 'TRACE') return 'debug';
	if (level === 'INFO') return 'info';
	return stream === 'stderr' ? 'error' : 'info';
}
export interface ProcessStats {
	cpu_percent: number;
	cpu_core_percent: number;
	logical_cores: number;
	memory_mb: number;
	disk_bytes_delta: number;
	sampled_at_ms: number;
	sample_interval_ms: number;
	ready: boolean;
}
export interface Sample {
	time: number;
	cpu: number | null;
	total: number | null;
	ram: number | null;
	disk: number | null;
}
export function sampleStats(s: ProcessStats | null, time = Date.now()): Sample {
	const valid = (n: number) => (Number.isFinite(n) && n >= 0 ? n : null);
	return {
		time: s?.sampled_at_ms ?? time,
		cpu: s?.ready ? valid(s.cpu_core_percent) : null,
		total: s?.ready ? valid(s.cpu_percent) : null,
		ram: s ? valid(s.memory_mb) : null,
		disk:
			s?.ready && s.sample_interval_ms > 0
				? valid((s.disk_bytes_delta * 1000) / s.sample_interval_ms / 1048576)
				: null
	};
}
export function appendSample(history: Sample[], sample: Sample): Sample[] {
	if (history.length && sample.time <= history[history.length - 1].time) return history;
	return [...history.filter((s) => s.time >= sample.time - 181000), sample].slice(-182);
}
export function chartPath(
	samples: Sample[],
	metric: Metric,
	cpuMode: 'cores' | 'total',
	now: number,
	seconds: number,
	ceiling: number
): string {
	let path = '';
	let previous: number | null = null;
	for (const s of samples) {
		const value = metric === 'cpu' ? (cpuMode === 'cores' ? s.cpu : s.total) : s[metric];
		if (s.time < now - seconds * 1000 || s.time > now || value === null) {
			previous = null;
			continue;
		}
		const x = ((s.time - (now - seconds * 1000)) / (seconds * 1000)) * 400;
		const y = 90 - Math.min(value / ceiling, 1) * 80;
		path += `${previous !== null && s.time - previous < 3500 ? 'L' : 'M'}${x.toFixed(2)},${y.toFixed(2)} `;
		previous = s.time;
	}
	return path;
}
