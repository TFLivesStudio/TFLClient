/** Logical pixels per second; every landscape travels to the right, never with the pointer. */
export function layerSpeed(depth: number, layerDepth: number) {
	return 0.6 + Math.max(0, Math.min(1, depth)) * Math.max(0, Math.min(1, layerDepth)) * 5.4;
}
export function sceneDelta(delta: number) {
	return Number.isFinite(delta) ? Math.max(0, Math.min(0.1, delta)) : 0;
}
export function advanceSceneTime(time: number, delta: number) {
	return (time + sceneDelta(delta)) % 86400;
}
export function layerOffset(
	time: number,
	depth: number,
	layerDepth: number,
	period: number,
	pixelAligned = false
) {
	const offset = (((time * layerSpeed(depth, layerDepth)) % period) + period) % period;
	return pixelAligned ? Math.floor(offset) : offset;
}
/** Independent bounded phase: no growing wall clock, no jump after a full cycle or long suspension. */
export function createLayerMotion(
	period: number,
	layerDepth: number,
	pixelAligned = false,
	initialPhase = 0
) {
	if (!Number.isFinite(period) || period <= 0) throw new Error('Invalid scene period');
	let phase = Number.isFinite(initialPhase) ? ((initialPhase % period) + period) % period : 0;
	return {
		advance(delta: number, depth: number) {
			phase = (phase + sceneDelta(delta) * layerSpeed(depth, layerDepth)) % period;
			return pixelAligned ? Math.floor(phase) : phase;
		},
		get offset() {
			return pixelAligned ? Math.floor(phase) : phase;
		},
		get phase() {
			return phase;
		}
	};
}
