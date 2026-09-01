<script setup lang="ts">
import { ref, watch, onUnmounted } from "vue";
import type { Status } from "./types";

const props = defineProps<{
    status: Status;
}>();

const elapsed = ref(0);
const show = ref(false);
let startTs: number | null = null;
let timer: number | undefined;
let showTimer: number | undefined;

// Wait for the shockwave pop before showing the timer, so it appears as the
// shockwave plays rather than instantly.
const SHOCKWAVE_DELAY_MS = 300;

function tick() {
    if (startTs === null) return;
    elapsed.value = Math.floor((Date.now() - startTs) / 1000);
}

watch(
    () => props.status,
    (s) => {
        if (s === "connected") {
            startTs = Date.now();
            elapsed.value = 0;
            tick();
            if (timer !== undefined) window.clearInterval(timer);
            timer = window.setInterval(tick, 1000);
            if (showTimer !== undefined) window.clearTimeout(showTimer);
            showTimer = window.setTimeout(() => {
                show.value = true;
            }, SHOCKWAVE_DELAY_MS);
        } else {
            show.value = false;
            startTs = null;
            if (timer !== undefined) {
                window.clearInterval(timer);
                timer = undefined;
            }
            if (showTimer !== undefined) {
                window.clearTimeout(showTimer);
                showTimer = undefined;
            }
        }
    },
    { immediate: true },
);

onUnmounted(() => {
    if (timer !== undefined) window.clearInterval(timer);
    if (showTimer !== undefined) window.clearTimeout(showTimer);
});

function format(secs: number): string {
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = secs % 60;
    const hh = String(h).padStart(2, "0");
    const mm = String(m).padStart(2, "0");
    const ss = String(s).padStart(2, "0");
    if (h > 0) {
        return `${hh}:${mm}:${ss}`;
    }
    return `${mm}:${ss}`;
}
</script>

<template>
    <span
        v-if="status === 'connected' && show"
        class="timer fixed inset-x-0 text-center timer-drop"
        style="top: calc(25vh + 208px)"
        >{{ format(elapsed) }}</span
    >
</template>

<style scoped>
.timer {
    font-family: "Product Sans", sans-serif;
    font-size: 1.25rem;
    color: #15b833;
    text-shadow: 2px 3px 6px rgba(0, 0, 0, 0.3);
    font-variant-numeric: tabular-nums;
}

/* Slide down from under the button with a fade, ~matching the shockwave. */
.timer-drop {
    transform-origin: center;
    animation: timer-drop 0.5s ease-out forwards;
}

@keyframes timer-drop {
    0% {
        opacity: 0;
        transform: translateY(-24px) scale(0.9);
    }
    60% {
        opacity: 1;
        transform: translateY(3px) scale(1);
    }
    100% {
        opacity: 1;
        transform: translateY(0) scale(1);
    }
}
</style>
