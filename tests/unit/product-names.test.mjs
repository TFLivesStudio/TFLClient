import { test } from 'node:test';
import assert from 'node:assert/strict';
import { clientDisplayName } from '../../src/lib/selection-names.ts';

test('display aliases preserve legacy catalog data and custom client names', () => {
	const target = Object.freeze({ displayName: 'TFL Selection 1.8.9', minecraftVersion: '1.8.9' });
	assert.equal(clientDisplayName(target.displayName, target.minecraftVersion), 'TFL Client 1.8.9');
	assert.equal(target.displayName, 'TFL Selection 1.8.9');
	assert.equal(clientDisplayName(undefined, '26.3'), 'TFL Client 26.3');
	assert.equal(clientDisplayName('TFLClient PvP', '1.8.9'), 'TFL Client PvP');
	assert.equal(clientDisplayName('Other client', '26.3'), 'Other client');
});
