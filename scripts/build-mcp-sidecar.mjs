import { execFileSync } from 'node:child_process';
import { mkdirSync, copyFileSync, statSync } from 'node:fs';
import path from 'node:path';
const profile = process.argv.includes('--review')
	? 'review'
	: process.argv.includes('--debug')
		? 'dev'
		: 'release';
const host = execFileSync('rustc', ['-vV'], { encoding: 'utf8' }).match(/^host: (.+)$/m)?.[1];
if (!host) throw new Error('Cannot determine Rust host target');
execFileSync(
	'cargo',
	['build', '--locked', '-p', 'tfl-mcp', '--bin', 'tflclient-mcp', '--profile', profile],
	{ stdio: 'inherit' }
);
const file = `tflclient-mcp${host.includes('windows') ? '.exe' : ''}`;
const source = path.join(
	process.env.CARGO_TARGET_DIR || 'target',
	profile === 'dev' ? 'debug' : profile,
	file
);
const output = path.join(
	'src-tauri',
	'binaries',
	`tflclient-mcp-${host}${host.includes('windows') ? '.exe' : ''}`
);
mkdirSync(path.dirname(output), { recursive: true });
copyFileSync(source, output);
console.log(`MCP helper: ${output} (${statSync(output).size} bytes)`);
