/** One requestAnimationFrame source for procedural scenes and character rendering. */
type Subscriber = { run: (delta: number, now: number) => void; fps: number; last: number };
const subscribers = new Set<Subscriber>();
let frame = 0;
function tick(now: number) {
	frame = 0;
	for (const s of subscribers) {
		const elapsed = now - s.last;
		if (elapsed + 0.5 >= 1000 / s.fps) {
			s.run(Math.min(elapsed / 1000, 0.1), now);
			s.last = now;
		}
	}
	if (subscribers.size) frame = requestAnimationFrame(tick);
}
export function subscribeFrame(run: Subscriber['run'], fps = 24) {
	const s = { run, fps: Math.max(1, Math.min(60, fps)), last: performance.now() };
	subscribers.add(s);
	if (!frame) frame = requestAnimationFrame(tick);
	return () => {
		subscribers.delete(s);
		if (!subscribers.size && frame) {
			cancelAnimationFrame(frame);
			frame = 0;
		}
	};
}
export function frameSubscriberCount() {
	return subscribers.size;
}
