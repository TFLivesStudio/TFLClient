<script lang="ts">
	import { onMount } from 'svelte';
	import { subscribeFrame } from './frameClock';
	let {
		fps = 24,
		dpr = 1,
		preset = 'aurora'
	}: { fps?: number; dpr?: number; preset?: string } = $props();
	let contextFailed = $state(false);
	let canvas: HTMLCanvasElement;
	onMount(() => {
		const gl = canvas.getContext('webgl', {
			alpha: true,
			antialias: false,
			depth: false,
			preserveDrawingBuffer: false
		});
		if (!gl) {
			contextFailed = true;
			return;
		}
		let stop = () => {};
		const shader = (type: number, source: string) => {
			const s = gl.createShader(type)!;
			gl.shaderSource(s, source);
			gl.compileShader(s);
			if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
				gl.deleteShader(s);
				throw Error('shader');
			}
			return s;
		};
		let program: WebGLProgram | null = null;
		let buffer: WebGLBuffer | null = null;
		try {
			const v = shader(
				gl.VERTEX_SHADER,
				'attribute vec2 a;void main(){gl_Position=vec4(a,0.,1.);}'
			);
			const f = shader(
				gl.FRAGMENT_SHADER,
				`precision mediump float;uniform float t;uniform vec2 r;uniform float biome;
float hash(vec2 p){return fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453);}
float noise(vec2 p){vec2 i=floor(p),f=fract(p);f=f*f*(3.-2.*f);return mix(mix(hash(i),hash(i+vec2(1,0)),f.x),mix(hash(i+vec2(0,1)),hash(i+vec2(1,1)),f.x),f.y);}
float fbm(vec2 p){float a=.5,v=0.;for(int i=0;i<4;i++){v+=a*noise(p);p=p*2.03+3.7;a*=.5;}return v;}
void main(){vec2 uv=gl_FragCoord.xy/r;vec2 q=vec2(uv.x*3.,uv.y*2.);float n=fbm(q+vec2(t*.018,-t*.012));float ribbon=0.;
for(int i=0;i<5;i++){float f=float(i);float wave=.64+sin(uv.x*3.2+t*.09+f*.35)*.12+sin(uv.x*8.-t*.06)*.025;float curtain=exp(-abs(uv.y-wave-f*.025)*(28.+f*4.));ribbon+=curtain*(.4+.6*noise(vec2(uv.x*42.+t*.1,f)));}
vec3 color=biome>2.5?vec3(.38,.93,.78):biome>1.5?vec3(.58,.37,.92):biome>.5?vec3(1.,.28,.07):vec3(.85,.89,.73);
float alpha=biome>2.5?ribbon*.075:biome>1.5?n*.055:biome>.5?n*.075*(1.-uv.y):n*.06;gl_FragColor=vec4(color*min(alpha,.25),min(alpha,.25));}`
			);
			program = gl.createProgram()!;
			gl.attachShader(program, v);
			gl.attachShader(program, f);
			gl.linkProgram(program);
			gl.deleteShader(v);
			gl.deleteShader(f);
			if (!gl.getProgramParameter(program, gl.LINK_STATUS)) throw Error('link');
			gl.useProgram(program);
			buffer = gl.createBuffer();
			gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
			gl.bufferData(
				gl.ARRAY_BUFFER,
				new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
				gl.STATIC_DRAW
			);
			const a = gl.getAttribLocation(program, 'a');
			gl.enableVertexAttribArray(a);
			gl.vertexAttribPointer(a, 2, gl.FLOAT, false, 0, 0);
			const t = gl.getUniformLocation(program, 't'),
				r = gl.getUniformLocation(program, 'r');
			gl.uniform1f(
				gl.getUniformLocation(program, 'biome'),
				preset === 'aurora' ? 3 : preset === 'void' ? 2 : preset === 'embers' ? 1 : 0
			);
			const started = performance.now();
			stop = subscribeFrame((_, now) => {
				const ratio = Math.min(devicePixelRatio || 1, dpr),
					width = Math.min(960, Math.round(canvas.clientWidth * ratio * 0.65)),
					height = Math.min(540, Math.round(canvas.clientHeight * ratio * 0.65));
				if (canvas.width !== width || canvas.height !== height) {
					canvas.width = width;
					canvas.height = height;
					gl.viewport(0, 0, width, height);
				}
				gl.uniform2f(r, width, height);
				gl.uniform1f(t, (now - started) / 1000);
				gl.drawArrays(gl.TRIANGLES, 0, 6);
			}, fps);
		} catch {
			contextFailed = true; /* Static scene remains visible. */
		}
		const lost = (e: Event) => {
			e.preventDefault();
			contextFailed = true;
			stop();
		};
		canvas.addEventListener('webglcontextlost', lost);
		return () => {
			stop();
			canvas.removeEventListener('webglcontextlost', lost);
			if (buffer) gl.deleteBuffer(buffer);
			if (program) gl.deleteProgram(program);
			gl.getExtension('WEBGL_lose_context')?.loseContext();
		};
	});
</script>

<canvas class:failed={contextFailed} bind:this={canvas} aria-hidden="true"></canvas>

<style>
	canvas.failed {
		opacity: 0;
	}
	canvas {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
	}
</style>
