import assert from 'node:assert/strict';
import test from 'node:test';
import {
	isSelectionInstance,
	instanceDisplayName,
	instanceEdition,
	selectionFallbackIcon
} from '../../src/lib/selection-identity.ts';

test('official presentation uses provenance independently of loader', () => {
	for (const loader of ['fabric', 'ornithe-gen2', 'forge', 'neoforge']) {
		const personal = { name: 'TFLClient personal', loader };
		assert.equal(isSelectionInstance(personal), false);
		assert.equal(instanceDisplayName(personal), personal.name);
		assert.equal(instanceEdition(personal), loader);
		const official = {
			...personal,
			name: 'TFLClient 26.3',
			selection: { target_id: 'fixture-target', release: '0.1.0-preview.4', channel: 'beta' }
		};
		assert.equal(isSelectionInstance(official), true);
		assert.equal(instanceDisplayName(official), 'TFL Client 26.3');
		assert.equal(instanceEdition(official), 'Preview 4 · Beta');
		assert.equal(selectionFallbackIcon(official.selection), '/images/tfl.svg');
	}
});
