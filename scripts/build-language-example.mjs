// Reproducible language-only .tfl fixture; hand-drawn small Brazilian flag, no remote assets.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { deflateSync } from 'node:zlib';
const root = resolve(import.meta.dirname, '..');
const base = resolve(root, 'examples/addons/portuguese-br');
const crc = (bytes) => {
	let c = 0xffffffff;
	for (const b of bytes) {
		c ^= b;
		for (let i = 0; i < 8; i++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
	}
	return (c ^ 0xffffffff) >>> 0;
};
const be = (value) => {
	const bytes = Buffer.alloc(4);
	bytes.writeUInt32BE(value);
	return bytes;
};
function chunk(type, bytes) {
	const data = Buffer.concat([Buffer.from(type), bytes]);
	return Buffer.concat([be(bytes.length), data, be(crc(data))]);
}
const width = 48,
	height = 32,
	pixels = Buffer.alloc((width * 4 + 1) * height);
for (let y = 0; y < height; y++)
	for (let x = 0; x < width; x++) {
		let color = [0, 156, 59];
		if (Math.abs(x - 23.5) / 20 + Math.abs(y - 15.5) / 12 < 1) color = [255, 223, 0];
		if ((x - 23.5) ** 2 + (y - 15.5) ** 2 < 65) color = [0, 39, 118];
		if ((x - 23.5) ** 2 + (y - 15.5) ** 2 < 65 && Math.abs(y - (13.2 + (x - 23.5) * 0.15)) < 0.65)
			color = [255, 255, 255];
		const i = y * (width * 4 + 1) + 1 + x * 4;
		pixels.set([...color, 255], i);
	}
const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(width);
ihdr.writeUInt32BE(height, 4);
ihdr[8] = 8;
ihdr[9] = 6;
mkdirSync(resolve(base, 'assets'), { recursive: true });
writeFileSync(
	resolve(base, 'assets/br.png'),
	Buffer.concat([
		Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
		chunk('IHDR', ihdr),
		chunk('IDAT', deflateSync(pixels)),
		chunk('IEND', Buffer.alloc(0))
	])
);
const manifest = JSON.parse(readFileSync(resolve(base, 'manifest.json'), 'utf8'));
const contract = JSON.parse(readFileSync(resolve(root, 'src/lib/i18n/catalog.v1.json'), 'utf8'));
for (const language of manifest.languages)
	for (const key of Object.keys(language.messages))
		if (!Object.hasOwn(contract.messages, key)) throw new Error(`Unknown example key: ${key}`);
const names = ['manifest.json', ...new Set(manifest.languages.map((language) => language.flag))];
const local = [],
	central = [];
let offset = 0;
for (const path of names) {
	const name = Buffer.from(path),
		bytes = readFileSync(resolve(base, path)),
		sum = crc(bytes),
		head = Buffer.alloc(30),
		row = Buffer.alloc(46);
	head.writeUInt32LE(0x04034b50);
	head.writeUInt16LE(20, 4);
	head.writeUInt32LE(sum, 14);
	head.writeUInt32LE(bytes.length, 18);
	head.writeUInt32LE(bytes.length, 22);
	head.writeUInt16LE(name.length, 26);
	local.push(head, name, bytes);
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
	resolve(root, 'examples/addons/portuguese-br.tfl'),
	Buffer.concat([...local, index, end])
);
console.log('examples/addons/portuguese-br.tfl');
