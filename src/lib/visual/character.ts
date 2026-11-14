/** Original joint choreography and bounded rigid-cape inertia; no cloth/geometry replacement. */
export type CharacterEvent = 'idle' | 'prepare' | 'launch' | 'success' | 'error';
export interface CharacterPose {
	headX: number;
	headY: number;
	bodyX: number;
	bodyY: number;
	bodyZ: number;
	leftArmX: number;
	leftArmZ: number;
	rightArmX: number;
	rightArmZ: number;
	leftLegX: number;
	rightLegX: number;
	lift: number;
}
const rest = (): CharacterPose => ({
	headX: 0,
	headY: 0,
	bodyX: 0,
	bodyY: 0,
	bodyZ: 0,
	leftArmX: 0,
	leftArmZ: 0.055,
	rightArmX: 0,
	rightArmZ: -0.055,
	leftLegX: 0,
	rightLegX: 0,
	lift: 0
});
export const clamp = (v: number, min: number, max: number) => Math.max(min, Math.min(max, v));
export interface Spring {
	value: number;
	velocity: number;
}
/** Semi-implicit spring, bounded delta/substeps for suspension and 15/24/60 Hz stability. */
export function stepSpring(s: Spring, target: number, delta: number, min: number, max: number) {
	const dt = clamp(Number.isFinite(delta) ? delta : 0, 0, 0.1);
	const steps = Math.max(1, Math.ceil(dt * 120)),
		h = dt / steps;
	for (let i = 0; i < steps; i++) {
		s.velocity += ((target - s.value) * 70 - s.velocity * 15) * h;
		s.value += s.velocity * h;
		if (s.value < min || s.value > max) {
			s.value = clamp(s.value, min, max);
			s.velocity = 0;
		}
	}
}
export class CharacterController {
	readonly pose = rest();
	readonly target = rest();
	yaw = 0;
	headTargetX = 0;
	headTargetY = 0;
	dragging = false;
	state: CharacterEvent = 'idle';
	cape: { pitch: Spring; roll: Spring } | null = null;
	capeSteps = 0;
	private previousYaw = 0;
	private previousLift = 0;
	private verticalSpeed = 0;
	private clock = 0;
	private gesture = 0;
	private gestureElapsed = 0;
	private lastHover = -20;
	get greeting() {
		return this.gesture > 0;
	}
	greet() {
		this.gesture = 1.7;
		this.gestureElapsed = 0;
	}
	hover() {
		if (this.clock - this.lastHover > 12) {
			this.lastHover = this.clock;
			this.greet();
		}
	}
	look(x: number, y: number) {
		this.headTargetY = clamp(x, -0.35, 0.35);
		this.headTargetX = clamp(y, -0.18, 0.18);
	}
	resetView() {
		this.yaw = 0;
		this.look(0, 0);
	}
	setEvent(event: CharacterEvent) {
		if (event !== this.state && event === 'success') this.greet();
		this.state = event;
	}
	step(delta: number, animated: boolean, hasCape: boolean, full: boolean) {
		const dt = clamp(Number.isFinite(delta) ? delta : 0, 0, 0.1);
		if (!animated) {
			Object.assign(this.pose, rest());
			this.gesture = 0;
			this.gestureElapsed = 0;
			this.cape = null;
			this.previousYaw = this.yaw;
			this.previousLift = 0;
			this.verticalSpeed = 0;
			return this.pose;
		}
		this.clock += dt;
		if (this.gesture > 0) {
			this.gesture -= dt;
			this.gestureElapsed += dt;
		}
		const p = this.target,
			t = this.clock;
		Object.assign(p, rest());
		p.headX = this.dragging ? 0 : this.headTargetX;
		p.headY = this.dragging ? 0 : this.headTargetY;
		p.lift = Math.sin(t * 1.5) * 0.035;
		p.leftArmZ += Math.sin(t * 1.3) * 0.008;
		p.rightArmZ -= Math.sin(t * 1.3) * 0.008;
		if (this.state === 'prepare' || this.state === 'launch') {
			const stride = Math.sin(t * 5) * (this.state === 'launch' ? 0.32 : 0.1);
			p.leftLegX = stride;
			p.rightLegX = -stride;
			p.leftArmX = -stride * 0.65;
			p.rightArmX = stride * 0.65;
			p.bodyX = 0.045;
		} else if (this.state === 'error') {
			p.headX = 0.12;
			p.bodyX = 0.025;
			p.headY = Math.sin(t * 7) * 0.07;
		}
		if (this.gesture > 0) {
			const envelope = Math.sin(Math.PI * clamp(this.gestureElapsed / 1.7, 0, 1));
			p.rightArmX = -2.3 * envelope;
			p.rightArmZ = -0.2 - Math.sin(this.gestureElapsed * 12) * 0.22 * envelope;
			p.bodyY = -0.035 * envelope;
			p.headY *= 0.4;
		}
		const blend = 1 - Math.exp(-dt * 12);
		for (const key of Object.keys(this.pose) as (keyof CharacterPose)[])
			this.pose[key] += (p[key] - this.pose[key]) * blend;
		if (!hasCape) {
			this.cape = null;
			return this.pose;
		}
		this.cape ??= { pitch: { value: 0.19, velocity: 0 }, roll: { value: 0, velocity: 0 } };
		const yawSpeed = dt > 0 ? clamp((this.yaw - this.previousYaw) / dt, -12, 12) : 0;
		const liftSpeed = dt > 0 ? (this.pose.lift - this.previousLift) / dt : 0;
		const acceleration = dt > 0 ? clamp((liftSpeed - this.verticalSpeed) / dt, -8, 8) : 0;
		const amplitude = full ? 1 : 0.45;
		const strideForce = Math.abs(this.pose.leftLegX) * 0.28;
		const pitch =
			0.19 +
			(Math.abs(yawSpeed) * 0.018 +
				strideForce +
				Math.abs(acceleration) * 0.009 +
				Math.abs(this.pose.bodyX) * 0.5) *
				amplitude;
		const roll = clamp(-yawSpeed * 0.025 * amplitude - this.pose.bodyY * 0.4, -0.18, 0.18);
		stepSpring(this.cape.pitch, pitch, dt, 0.13, 0.65);
		stepSpring(this.cape.roll, roll, dt, -0.2, 0.2);
		this.capeSteps++;
		this.previousYaw = this.yaw;
		this.previousLift = this.pose.lift;
		this.verticalSpeed = liftSpeed;
		return this.pose;
	}
}
