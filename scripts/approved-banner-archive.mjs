// Inspect approved image archives without trusting ZIP paths, modes or declared sizes.
import { readFile } from 'node:fs/promises';
import { inflateRawSync } from 'node:zlib';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
const MAX_ARCHIVE = 256 * 1024 * 1024,
	MAX_IMAGE = 32 * 1024 * 1024;
const fail = (message) => {
	throw new Error(`INVALID_BANNER_ARCHIVE: ${message}`);
};
export function safeArchivePath(name) {
	if (
		!name ||
		name.length > 240 ||
		[...name].some((c) => c.charCodeAt(0) < 32 || c.charCodeAt(0) === 127 || c === ':') ||
		/^[\\/]/.test(name)
	)
		fail('unsafe path');
	const normalized = name.replaceAll('\\', '/');
	const parts = normalized.replace(/\/$/, '').split('/');
	if (
		parts.some(
			(p) =>
				!p ||
				p === '.' ||
				p === '..' ||
				/[. ]$/.test(p) ||
				/^(?:con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(p)
		)
	)
		fail('unsafe path component');
	return normalized;
}
export function crc32(bytes) {
	let crc = 0xffffffff;
	for (const value of bytes) {
		crc ^= value;
		for (let i = 0; i < 8; i++) crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
	}
	return (crc ^ 0xffffffff) >>> 0;
}
function dimensions(bytes, extension) {
	if (
		extension === 'png' &&
		bytes.length >= 24 &&
		bytes.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10])) &&
		bytes.toString('ascii', 12, 16) === 'IHDR'
	)
		return [bytes.readUInt32BE(16), bytes.readUInt32BE(20)];
	if ((extension === 'jpg' || extension === 'jpeg') && bytes[0] === 255 && bytes[1] === 216) {
		let at = 2;
		while (at + 4 < bytes.length) {
			if (bytes[at++] !== 255) break;
			while (bytes[at] === 255) at++;
			const marker = bytes[at++];
			if (marker === 216 || marker === 217) continue;
			const length = bytes.readUInt16BE(at);
			if (length < 2 || at + length > bytes.length) break;
			if (
				[192, 193, 194, 195, 197, 198, 199, 201, 202, 203, 205, 206, 207].includes(marker) &&
				length >= 7
			)
				return [bytes.readUInt16BE(at + 5), bytes.readUInt16BE(at + 3)];
			at += length;
		}
	}
	if (
		extension === 'webp' &&
		bytes.length >= 30 &&
		bytes.toString('ascii', 0, 4) === 'RIFF' &&
		bytes.toString('ascii', 8, 12) === 'WEBP'
	) {
		const kind = bytes.toString('ascii', 12, 16);
		if (kind === 'VP8X') return [1 + bytes.readUIntLE(24, 3), 1 + bytes.readUIntLE(27, 3)];
		if (kind === 'VP8 ' && bytes[23] === 157 && bytes[24] === 1 && bytes[25] === 42)
			return [bytes.readUInt16LE(26) & 16383, bytes.readUInt16LE(28) & 16383];
		if (kind === 'VP8L' && bytes[20] === 47) {
			const bits = bytes.readUInt32LE(21);
			return [(bits & 16383) + 1, ((bits >>> 14) & 16383) + 1];
		}
	}
	fail('image signature or dimensions do not match extension');
}
export function inspectBannerZip(data) {
	if (!Buffer.isBuffer(data) || data.length > MAX_ARCHIVE || data.length < 22) fail('archive size');
	let end = -1;
	for (let at = data.length - 22; at >= Math.max(0, data.length - 65557); at--)
		if (
			data.readUInt32LE(at) === 0x06054b50 &&
			at + 22 + data.readUInt16LE(at + 20) === data.length
		) {
			end = at;
			break;
		}
	if (end < 0) fail('missing ZIP directory');
	const count = data.readUInt16LE(end + 10),
		length = data.readUInt32LE(end + 12),
		offset = data.readUInt32LE(end + 16);
	if (
		data.readUInt16LE(end + 4) ||
		data.readUInt16LE(end + 6) ||
		data.readUInt16LE(end + 8) !== count ||
		count > 1000 ||
		count === 65535 ||
		offset + length !== end
	)
		fail('split, ZIP64 or malformed directory');
	let at = offset,
		total = 0;
	const names = new Set(),
		ranges = [],
		images = [];
	for (let entry = 0; entry < count; entry++) {
		if (at + 46 > end || data.readUInt32LE(at) !== 0x02014b50) fail('central entry');
		const flags = data.readUInt16LE(at + 8),
			method = data.readUInt16LE(at + 10),
			crc = data.readUInt32LE(at + 16),
			compressed = data.readUInt32LE(at + 20),
			size = data.readUInt32LE(at + 24),
			nameLength = data.readUInt16LE(at + 28),
			extra = data.readUInt16LE(at + 30),
			comment = data.readUInt16LE(at + 32),
			external = data.readUInt32LE(at + 38),
			local = data.readUInt32LE(at + 42);
		if (
			at + 46 + nameLength + extra + comment > end ||
			flags & 1 ||
			![0, 8].includes(method) ||
			size > MAX_IMAGE ||
			compressed > MAX_IMAGE ||
			(size > 1024 * 1024 && size / Math.max(1, compressed) > 250)
		)
			fail('unsupported or excessive entry');
		const rawName = data.subarray(at + 46, at + 46 + nameLength);
		let name;
		try {
			name = safeArchivePath(new TextDecoder('utf-8', { fatal: true }).decode(rawName));
		} catch {
			fail('invalid filename');
		}
		const mode = (external >>> 16) & 0xf000;
		if (mode && mode !== 0x8000 && mode !== 0x4000) fail('non-regular file');
		const key = name.toLowerCase();
		if (names.has(key)) fail('duplicate or case-colliding entry');
		names.add(key);
		total += size;
		if (total > MAX_ARCHIVE) fail('expanded size');
		if (local + 30 > offset || data.readUInt32LE(local) !== 0x04034b50) fail('local header');
		const localName = data.readUInt16LE(local + 26),
			localExtra = data.readUInt16LE(local + 28),
			start = local + 30 + localName + localExtra;
		if (
			data.readUInt16LE(local + 6) !== flags ||
			data.readUInt16LE(local + 8) !== method ||
			!data.subarray(local + 30, local + 30 + localName).equals(rawName) ||
			start + compressed > offset
		)
			fail('entry bounds or local mismatch');
		ranges.push([local, start + compressed]);
		if (!name.endsWith('/')) {
			const extension = name.split('.').pop().toLowerCase();
			if (!['png', 'jpg', 'jpeg', 'webp'].includes(extension)) fail('unexpected file type');
			let bytes;
			try {
				bytes =
					method === 0
						? data.subarray(start, start + compressed)
						: inflateRawSync(data.subarray(start, start + compressed), {
								maxOutputLength: MAX_IMAGE
							});
			} catch {
				fail('invalid compression');
			}
			if (bytes.length !== size || crc32(bytes) !== crc) fail('size or CRC mismatch');
			const [width, height] = dimensions(bytes, extension);
			if (!width || !height || width > 16384 || height > 16384 || width * height > 64 * 1024 * 1024)
				fail('image dimensions exceed bounds');
			images.push({
				name,
				extension,
				width,
				height,
				size,
				sha256: createHash('sha256').update(bytes).digest('hex'),
				bytes
			});
		} else if (size !== 0) fail('directory has data');
		at += 46 + nameLength + extra + comment;
	}
	if (at !== end || !images.length) fail('directory length or no images');
	ranges.sort((a, b) => a[0] - b[0]);
	for (let i = 1; i < ranges.length; i++)
		if (ranges[i][0] < ranges[i - 1][1]) fail('overlapping entries');
	return { sha256: createHash('sha256').update(data).digest('hex'), images };
}
export async function inspectBannerArchive(file) {
	return inspectBannerZip(await readFile(file));
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
	const result = await inspectBannerArchive(process.argv[2]);
	console.log(
		JSON.stringify(
			{
				...result,
				images: result.images.map((entry) => ({
					name: entry.name,
					sha256: entry.sha256,
					width: entry.width,
					height: entry.height,
					format: entry.format
				}))
			},
			null,
			2
		)
	);
}
