<script lang="ts">
	import { onMount } from 'svelte';
	import { subscribeFrame } from './frameClock';
	let { fps = 24, dpr = 1 }: { fps?: number; dpr?: number } = $props();
	let canvas: HTMLCanvasElement;
	onMount(() => {
		const gl = canvas.getContext('webgl', {
			alpha: true,
			antialias: false,
			depth: false,
			preserveDrawingBuffer: false
		});
		if (!gl) return;
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
				'precision mediump float;uniform float t;uniform vec2 r;void main(){vec2 uv=gl_FragCoord.xy/r;float wave=sin(uv.x*6.+t*.25)*.08+sin(uv.x*12.-t*.18)*.04;float glow=exp(-abs(uv.y-.6-wave)*16.);gl_FragColor=vec4(.15,.8,.6,glow*.16);}'
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
			const started = performance.now();
			stop = subscribeFrame((_, now) => {
				const ratio = Math.min(devicePixelRatio || 1, dpr),
					width = Math.min(1600, Math.round(canvas.clientWidth * ratio)),
					height = Math.min(900, Math.round(canvas.clientHeight * ratio));
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
			/* Static scene underneath remains usable on unsupported GPUs. */
		}
		const lost = (e: Event) => {
			e.preventDefault();
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

<canvas bind:this={canvas} aria-hidden="true"></canvas>

<style>
	canvas {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
	}
</style>
