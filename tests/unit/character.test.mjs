import { test } from 'node:test';
import assert from 'node:assert/strict';
import { CharacterController, stepSpring } from '../../src/lib/visual/character.ts';
test('idle, real events and greetings never rotate the body orientation automatically', () => {
	const c = new CharacterController();
	for (const event of ['idle', 'prepare', 'launch', 'success', 'error']) {
		c.setEvent(event);
		for (let i = 0; i < 300; i++) c.step(1 / 60, true, false, true);
		assert.equal(c.yaw, 0);
	}
	c.yaw = 2;
	for (let i = 0; i < 300; i++) c.step(1 / 60, true, false, true);
	assert.equal(c.yaw, 2, 'explicit user rotation is retained');
	c.resetView();
	assert.equal(c.yaw, 0);
});
test('head tracking is clamped, interpolates and returns to neutral', () => {
	const c = new CharacterController();
	c.look(100, -100);
	c.step(1 / 60, true, false, true);
	assert.ok(c.pose.headY > 0 && c.pose.headY < 0.35);
	assert.ok(c.pose.headX < 0 && c.pose.headX > -0.18);
	for (let i = 0; i < 120; i++) c.step(1 / 60, true, false, true);
	assert.ok(c.pose.headY <= 0.35 && c.pose.headX >= -0.18);
	c.look(0, 0);
	for (let i = 0; i < 120; i++) c.step(1 / 60, true, false, true);
	assert.ok(Math.abs(c.pose.headY) < 0.0001 && Math.abs(c.pose.headX) < 0.0001);
});
test('greeting blends poses, completes and hover acknowledgement has a long cooldown', () => {
	const c = new CharacterController();
	c.hover();
	c.step(1 / 24, true, false, false);
	const first = c.pose.rightArmX;
	assert.ok(first < 0 && first > -0.2, 'no disconnected clip jump');
	for (let i = 0; i < 100; i++) c.step(1 / 24, true, false, false);
	assert.equal(c.greeting, false);
	assert.ok(Math.abs(c.pose.rightArmX) < 0.001);
	c.hover();
	assert.equal(c.greeting, false, 'no repetitive hover greeting');
	c.greet();
	assert.equal(c.greeting, true, 'explicit activation remains available');
});
test('no equipped cape and reduced motion perform zero cape physics', () => {
	const c = new CharacterController();
	for (let i = 0; i < 300; i++) {
		c.yaw += 0.03;
		c.step(1 / 60, true, false, true);
	}
	assert.equal(c.capeSteps, 0);
	assert.equal(c.cape, null);
	for (let i = 0; i < 60; i++) c.step(1 / 60, false, true, true);
	assert.equal(c.capeSteps, 0);
	assert.equal(c.pose.lift, 0);
	assert.equal(c.pose.headY, 0);
	c.greet();
	c.step(0, false, true, true);
	assert.equal(c.greeting, false, 'suspension cancels stale gestures');
});
test('cape is an inertial damped spring, bounded under rapid drag and settles after release', () => {
	for (const fps of [15, 24, 60]) {
		const c = new CharacterController();
		for (let i = 0; i < fps; i++) {
			c.yaw += 0.1;
			c.step(1 / fps, true, true, true);
		}
		assert.ok(c.cape.pitch.value > 0.19);
		assert.ok(c.cape.roll.value < 0);
		for (let i = 0; i < fps * 10; i++) c.step(1 / fps, true, true, true);
		assert.ok(Math.abs(c.cape.roll.value) < 0.001);
		assert.ok(c.cape.pitch.value >= 0.13 && c.cape.pitch.value <= 0.65);
		assert.ok(Math.abs(c.cape.pitch.velocity) < 0.01);
	}
	const s = { value: 0.2, velocity: 100 };
	stepSpring(s, 50, 100, 0.13, 0.65);
	assert.ok(s.value >= 0.13 && s.value <= 0.65);
});
