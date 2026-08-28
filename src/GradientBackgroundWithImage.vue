<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import type { Status } from "./types";

const props = defineProps<{
    status: Status;
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);

const colors: Record<Status, [number, number, number]> = {
    disconnected: [0x28, 0x00, 0x00],
    connecting: [0x1d, 0x1a, 0x03],
    connected: [0x08, 0x2b, 0x09],
};

const TRANSITION_MS = 2150;

const VERT = `#version 300 es
layout(location = 0) in vec2 aPos;
void main() {
    gl_Position = vec4(aPos, 0.0, 1.0);
}
`;

const FRAG = `#version 300 es
precision highp float;

uniform vec3 uFromColor;
uniform vec3 uTargetColor;
uniform float uProgress;
uniform float uTime;
uniform vec2 uResolution;
out vec4 fragColor;

void main() {
    // Normalized coordinates in [0,1].
    vec2 uv = gl_FragCoord.xy / uResolution;

    // Subtle rotation of the gradient direction over time.
    float angle = uTime * 0.05;
    float s = sin(angle);
    float c = cos(angle);
    vec2 rot = vec2(
        (uv.x - 0.5) * c - (uv.y - 0.5) * s,
        (uv.x - 0.5) * s + (uv.y - 0.5) * c
    ) + 0.5;

    // Diagonal (top-left -> bottom-right) gradient factor in [0,1].
    float t = (rot.x + (1.0 - rot.y)) * 0.5;

    // Current color is the CPU-interpolated mix of from -> target.
    vec3 current = mix(uFromColor, uTargetColor, uProgress);

    // Top-left is black, bottom-right reaches the current color.
    vec3 color = mix(vec3(0.0), current, t);

    // Dither to hide 8-bit output quantization.
    float d = fract(sin(dot(gl_FragCoord.xy, vec2(12.9898, 78.233))) * 43758.5453);
    color += (d - 0.5) / 255.0;

    fragColor = vec4(color, 1.0);
}
`;

interface Ctx {
    gl: WebGL2RenderingContext;
    uFromColor: WebGLUniformLocation;
    uTargetColor: WebGLUniformLocation;
    uProgress: WebGLUniformLocation;
    uTime: WebGLUniformLocation;
    uResolution: WebGLUniformLocation;
    from: [number, number, number];
    target: [number, number, number];
    progress: number;
    raf: number;
    prevTime: number;
}

let ctx: Ctx | null = null;

function compileShader(
    gl: WebGL2RenderingContext,
    type: number,
    source: string,
): WebGLShader {
    const shader = gl.createShader(type)!;
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        throw new Error(gl.getShaderInfoLog(shader) ?? "shader compile error");
    }
    return shader;
}

function pushColor(loc: WebGLUniformLocation, c: [number, number, number]) {
    ctx!.gl.uniform3f(loc, c[0] / 255, c[1] / 255, c[2] / 255);
}

function render() {
    const c = ctx;
    if (!c) return;
    c.gl.drawArrays(c.gl.TRIANGLES, 0, 3);
}

function frame(now: number) {
    const c = ctx;
    if (!c) return;

    const dt = now - c.prevTime;
    c.prevTime = now;

    if (c.progress < 1) {
        c.progress = Math.min(1, c.progress + dt / TRANSITION_MS);
        const eased = 1 - Math.pow(1 - c.progress, 3);
        c.gl.uniform1f(c.uProgress, eased);
    }

    c.gl.uniform1f(c.uTime, now / 1000);
    render();
    c.raf = requestAnimationFrame(frame);
}

function init() {
    const canvas = canvasRef.value;
    if (!canvas) return;
    const gl = canvas.getContext("webgl2", { antialias: true });
    if (!gl) return;

    const vs = compileShader(gl, gl.VERTEX_SHADER, VERT);
    const fs = compileShader(gl, gl.FRAGMENT_SHADER, FRAG);
    const program = gl.createProgram()!;
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        throw new Error(gl.getProgramInfoLog(program) ?? "link error");
    }
    gl.useProgram(program);

    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(
        gl.ARRAY_BUFFER,
        new Float32Array([-1, -1, 3, -1, -1, 3]),
        gl.STATIC_DRAW,
    );
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

    const target = colors[props.status];
    ctx = {
        gl,
        uFromColor: gl.getUniformLocation(program, "uFromColor")!,
        uTargetColor: gl.getUniformLocation(program, "uTargetColor")!,
        uProgress: gl.getUniformLocation(program, "uProgress")!,
        uTime: gl.getUniformLocation(program, "uTime")!,
        uResolution: gl.getUniformLocation(program, "uResolution")!,
        from: target,
        target,
        progress: 1,
        raf: 0,
        prevTime: performance.now(),
    };

    pushColor(ctx.uFromColor, ctx.from);
    pushColor(ctx.uTargetColor, ctx.target);
    gl.uniform1f(ctx.uProgress, 1);

    resize();
    ctx.raf = requestAnimationFrame(frame);
}

function setTarget(status: Status) {
    if (!ctx) return;
    ctx.from = ctx.target;
    ctx.target = colors[status];
    pushColor(ctx.uFromColor, ctx.from);
    pushColor(ctx.uTargetColor, ctx.target);
    ctx.progress = 0;
}

function resize() {
    const canvas = canvasRef.value;
    if (!canvas || !ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const w = Math.round(canvas.clientWidth * dpr);
    const h = Math.round(canvas.clientHeight * dpr);
    if (canvas.width !== w || canvas.height !== h) {
        canvas.width = w;
        canvas.height = h;
        ctx.gl.viewport(0, 0, w, h);
    }
    ctx.gl.uniform2f(ctx.uResolution, w, h);
    render();
}

onMounted(() => {
    init();
    window.addEventListener("resize", resize);
});

onUnmounted(() => {
    window.removeEventListener("resize", resize);
    if (ctx) {
        cancelAnimationFrame(ctx.raf);
        ctx = null;
    }
});

watch(
    () => props.status,
    (s) => setTarget(s),
);
</script>

<template>
    <div class="gradient-bg">
        <canvas
            ref="canvasRef"
            class="gradient-canvas"
            width="640"
            height="640"
        ></canvas>
        <img src="./assets/world.svg" alt="" class="world" />
    </div>
</template>

<style scoped>
.gradient-bg {
    position: absolute;
    inset: 0;
    overflow: hidden;
    background: transparent;
}

.gradient-canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: block;
    border-radius: 12px;
    overflow: hidden;
}

.world {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    height: 400px;
    width: 450px;
    box-sizing: border-box;
    pointer-events: none;
    border-radius: 12px;
    overflow: hidden;
}
</style>
