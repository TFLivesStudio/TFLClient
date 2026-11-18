import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
	CharacterController,
	stepSpring,
	idleGlance,
	applyCharacterPose
} from '../../src/lib/visual/character.ts';
import { PlayerObject } from 'skinview3d';
import { Group } from 'three';
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
	assert.ok(
		c.pose.rightArmX > -0.04 && c.pose.rightArmX < -0.01,
		'returns to the relaxed breathing pose'
	);
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
		c.look(0, 0); // Explicit neutral tracking isolates spring settling from autonomous glances.
		for (let i = 0; i < fps * 10; i++) c.step(1 / fps, true, true, true);
		assert.ok(Math.abs(c.cape.roll.value) < 0.001);
		assert.ok(c.cape.pitch.value >= 0.13 && c.cape.pitch.value <= 0.65);
		assert.ok(Math.abs(c.cape.pitch.velocity) < 0.01);
	}
	const s = { value: 0.2, velocity: 100 };
	stepSpring(s, 50, 100, 0.13, 0.65);
	assert.ok(s.value >= 0.13 && s.value <= 0.65);
});

test('idle breath uses rigid torso and shoulders, with quiet alternating glances and no body bob', () => {
	const c = new CharacterController();
	let minHead = 0,
		maxHead = 0,
		moving = 0,
		still = 0,
		torso = 0;
	for (let i = 0; i < 60 * 53; i++) {
		const p = c.step(1 / 60, true, false, true);
		minHead = Math.min(minHead, p.headY);
		maxHead = Math.max(maxHead, p.headY);
		torso = Math.max(torso, Math.abs(p.bodyX));
		if (Math.abs(p.headY) > 0.05) moving++;
		else still++;
		assert.equal(p.lift, 0, 'idle never moves the whole body up/down');
		assert.ok(Math.abs(p.headY) <= 0.27 && Math.abs(p.bodyY) <= 0.013);
		assert.equal(p.leftLegX, 0);
		assert.equal(p.rightLegX, 0);
	}
	assert.ok(minHead < -0.2 && maxHead > 0.2, 'looks to both sides');
	assert.ok(still > moving * 2, 'natural pauses dominate movement');
	assert.ok(torso > 0.005 && torso < 0.008, 'small orientation-only breath');
	assert.deepEqual(idleGlance(0), { yaw: 0, anticipation: 0 });
});

test('pointer tracking and launch gestures outrank autonomous idle and resume smoothly', () => {
	const c = new CharacterController();
	for (let i = 0; i < 9 * 60; i++) c.step(1 / 60, true, false, true);
	assert.ok(c.pose.headY < -0.2);
	c.look(0.1, 0.05);
	for (let i = 0; i < 120; i++) c.step(1 / 60, true, false, true);
	assert.ok(Math.abs(c.pose.headY - 0.1) < 0.0001, 'tracking owns head yaw');
	c.greet();
	c.setEvent('launch');
	for (let i = 0; i < 60; i++) {
		const p = c.step(1 / 60, true, false, true);
		assert.ok(Math.abs(p.rightArmX) < 0.3, 'queued greeting cannot overwrite launch stride');
	}
	c.step(100, false, true, true);
	assert.equal(c.greeting, false);
	assert.equal(c.pose.bodyX, 0);
	c.releaseLook();
	c.setEvent('idle');
	c.step(100, true, true, true);
	for (const value of Object.values(c.pose)) assert.ok(Number.isFinite(value));
	assert.ok(Math.abs(c.pose.headY) < 0.27, 'resume is delta-bounded and blended');
});

test('classic/slim use the original six joints, rigid geometry and original skin layers', () => {
	for (const model of ['default', 'slim']) {
		const playerObject = new PlayerObject();
		playerObject.skin.modelType = model;
		const playerWrapper = new Group();
		playerWrapper.add(playerObject);
		const original = [];
		playerObject.skin.traverse((part) =>
			original.push({
				part,
				position: part.position.toArray(),
				scale: part.scale.toArray(),
				geometry: part.geometry,
				children: [...part.children]
			})
		);
		const c = new CharacterController();
		for (let i = 0; i < 1200; i++) {
			c.step(1 / 60, true, true, true);
			applyCharacterPose({ playerObject, playerWrapper }, c);
		}
		assert.deepEqual(
			playerObject.skin.children.map((part) => part.name),
			['head', 'body', 'rightArm', 'leftArm', 'rightLeg', 'leftLeg']
		);
		for (const { part, position, scale, geometry, children } of original) {
			assert.deepEqual(part.position.toArray(), position, 'original joint pivot');
			assert.deepEqual(part.scale.toArray(), scale, 'no chest or limb scaling');
			assert.equal(part.geometry, geometry, 'mesh geometry is untouched');
			assert.deepEqual(part.children, children, 'no replacement skeleton');
		}
		assert.equal(playerObject.skin.modelType, model);
		assert.equal(playerWrapper.rotation.y, 0);
		playerObject.traverse((part) => {
			part.geometry?.dispose();
			if (Array.isArray(part.material)) part.material.forEach((m) => m.dispose());
			else part.material?.dispose();
		});
	}
});
