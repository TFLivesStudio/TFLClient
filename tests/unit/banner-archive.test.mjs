import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
	crc32,
	inspectBannerZip,
	safeArchivePath
} from '../../scripts/approved-banner-archive.mjs';
const png = Buffer.from(
	'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO5N9XcAAAAASUVORK5CYII=',
	'base64'
);
function zip(entries) {
	const locals = [],
		directory = [];
	let offset = 0;
	for (const { name, bytes = png, mode = 0x81a40000 } of entries) {
		const filename = Buffer.from(name),
			local = Buffer.alloc(30),
			central = Buffer.alloc(46);
		const crc = crc32(bytes);
		local.writeUInt32LE(0x04034b50);
		local.writeUInt16LE(20, 4);
		local.writeUInt32LE(crc, 14);
		local.writeUInt32LE(bytes.length, 18);
		local.writeUInt32LE(bytes.length, 22);
		local.writeUInt16LE(filename.length, 26);
		central.writeUInt32LE(0x02014b50);
		central.writeUInt16LE(20, 6);
		central.writeUInt32LE(crc, 16);
		central.writeUInt32LE(bytes.length, 20);
		central.writeUInt32LE(bytes.length, 24);
		central.writeUInt16LE(filename.length, 28);
		central.writeUInt32LE(mode >>> 0, 38);
		central.writeUInt32LE(offset, 42);
		const item = Buffer.concat([local, filename, bytes]);
		locals.push(item);
		directory.push(Buffer.concat([central, filename]));
		offset += item.length;
	}
	const central = Buffer.concat(directory),
		end = Buffer.alloc(22);
	end.writeUInt32LE(0x06054b50);
	end.writeUInt16LE(entries.length, 8);
	end.writeUInt16LE(entries.length, 10);
	end.writeUInt32LE(central.length, 12);
	end.writeUInt32LE(offset, 16);
	return Buffer.concat([...locals, central, end]);
}
test('approved archive inspection reports dimensions and hashes without extracting files', () => {
	const result = inspectBannerZip(zip([{ name: 'Minecraft 1.8/banner.png' }]));
	assert.match(result.sha256, /^[a-f0-9]{64}$/);
	assert.equal(result.images[0].width, 1);
	assert.equal(result.images[0].height, 1);
	assert.ok(result.images[0].bytes.equals(png));
});
test('archive paths reject traversal, absolute names, alternate streams and Windows reserved names', () => {
	for (const name of [
		'../bad.png',
		'a/../../bad.png',
		'C:/bad.png',
		'/bad.png',
		'\\\\host\\bad.png',
		'a:stream.png',
		'CON.png',
		'folder/NUL.webp',
		'a./bad.png',
		'x\u0000.png'
	]) {
		assert.throws(() => safeArchivePath(name), /INVALID_BANNER_ARCHIVE/);
		assert.throws(() => inspectBannerZip(zip([{ name }])), /INVALID_BANNER_ARCHIVE/);
	}
});
test('archives reject symlinks, case collisions, unexpected files, bad checksums and malformed image headers', () => {
	for (const input of [
		zip([{ name: 'a.png', mode: 0xa1ff0000 }]),
		zip([{ name: 'a.png' }, { name: 'A.png' }]),
		zip([{ name: 'run.exe' }]),
		zip([{ name: 'a.png', bytes: Buffer.from('not an image') }])
	])
		assert.throws(() => inspectBannerZip(input), /INVALID_BANNER_ARCHIVE/);
	const bad = zip([{ name: 'a.png' }]);
	bad[36] ^= 1;
	assert.throws(() => inspectBannerZip(bad), /INVALID_BANNER_ARCHIVE/);
	const huge = Buffer.from(png);
	huge.writeUInt32BE(20000, 16);
	assert.throws(() => inspectBannerZip(zip([{ name: 'a.png', bytes: huge }])), /dimensions/);
});
