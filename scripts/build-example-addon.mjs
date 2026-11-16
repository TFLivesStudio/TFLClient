// Original periodic pixel art, PNG encoder and stored ZIP. No external dependency.
// Usage: node scripts/build-example-addon.mjs
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { deflateSync } from 'node:zlib';
const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const base = join(root, 'examples/addons/orange-dunes');
const crc = (bytes) => {
	let c = 0xffffffff;
	for (const b of bytes) {
		c ^= b;
		for (let i = 0; i < 8; i++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
	}
	return (c ^ 0xffffffff) >>> 0;
};
const be = (n) => {
	const b = Buffer.alloc(4);
	b.writeUInt32BE(n);
	return b;
};
const chunk = (type, data) => {
	const body = Buffer.concat([Buffer.from(type), data]);
	return Buffer.concat([be(data.length), body, be(crc(body))]);
};
function image(level, color) {
	const width = 384,
		height = 216,
		raw = Buffer.alloc((width * 4 + 1) * height);
	for (let y = 0; y < height; y++)
		for (let x = 0; x < width; x++) {
			const ridge = Math.round(
				level +
					Math.sin((x / width) * Math.PI * 2) * 12 +
					Math.sin((x / width) * Math.PI * 4 + level) * 5
			);
			const index = y * (width * 4 + 1) + 1 + x * 4;
			if (y >= ridge) {
				raw[index] = color[0];
				raw[index + 1] = color[1];
				raw[index + 2] = color[2];
				raw[index + 3] = 255;
			}
		}
	const ihdr = Buffer.alloc(13);
	ihdr.writeUInt32BE(width);
	ihdr.writeUInt32BE(height, 4);
	ihdr[8] = 8;
	ihdr[9] = 6;
	return Buffer.concat([
		Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
		chunk('IHDR', ihdr),
		chunk('IDAT', deflateSync(raw)),
		chunk('IEND', Buffer.alloc(0))
	]);
}
mkdirSync(join(base, 'assets'), { recursive: true });
for (const [name, level, color] of [
	['distant', 92, [78, 51, 74]],
	['ridge', 136, [128, 68, 72]],
	['dunes', 177, [187, 100, 66]]
])
	writeFileSync(join(base, `assets/${name}.png`), image(level, color));
const manifest = JSON.parse(readFileSync(join(base, 'manifest.json'), 'utf8'));
const names = [
	'manifest.json',
	...new Set(manifest.scenes.flatMap((s) => s.layers.map((l) => l.path)))
];
const local = [],
	central = [];
let offset = 0;
for (const path of names) {
	const name = Buffer.from(path),
		bytes = readFileSync(join(base, path)),
		sum = crc(bytes);
	const head = Buffer.alloc(30);
	head.writeUInt32LE(0x04034b50);
	head.writeUInt16LE(20, 4);
	head.writeUInt32LE(sum, 14);
	head.writeUInt32LE(bytes.length, 18);
	head.writeUInt32LE(bytes.length, 22);
	head.writeUInt16LE(name.length, 26);
	local.push(head, name, bytes);
	const row = Buffer.alloc(46);
	row.writeUInt32LE(0x02014b50);
	row.writeUInt16LE(20, 4);
	row.writeUInt16LE(20, 6);
	row.writeUInt32LE(sum, 16);
	row.writeUInt32LE(bytes.length, 20);
	row.writeUInt32LE(bytes.length, 24);
	row.writeUInt16LE(name.length, 28);
	row.writeUInt32LE(offset, 42);
	central.push(row, name);
	offset += head.length + name.length + bytes.length;
}
const index = Buffer.concat(central),
	end = Buffer.alloc(22);
end.writeUInt32LE(0x06054b50);
end.writeUInt16LE(names.length, 8);
end.writeUInt16LE(names.length, 10);
end.writeUInt32LE(index.length, 12);
end.writeUInt32LE(offset, 16);
writeFileSync(
	join(root, 'examples/addons/orange-dunes.tfl'),
	Buffer.concat([...local, index, end])
);
process.stdout.write('examples/addons/orange-dunes.tfl\n');
