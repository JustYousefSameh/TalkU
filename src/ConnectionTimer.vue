<script setup lang="ts">
import { ref, watch, onUnmounted } from "vue";
import type { Status } from "./types";

const props = defineProps<{
    status: Status;
}>();

const elapsed = ref(0);
let startTs: number | null = null;
let timer: number | undefined;

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
        } else {
            startTs = null;
            if (timer !== undefined) {
                window.clearInterval(timer);
                timer = undefined;
            }
        }
    },
    { immediate: true },
);

onUnmounted(() => {
    if (timer !== undefined) window.clearInterval(timer);
});

function format(secs: number): string {
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = secs % 60;
    const mm = String(m).padStart(2, "0");
    const ss = String(s).padStart(2, "0");
    if (h > 0) {
        return `${h}:${mm}:${ss}`;
    }
    return `${mm}:${ss}`;
}
</script>

<template>
    <span
        v-if="status === 'connected'"
        class="timer fixed inset-x-0 text-center"
        style="bottom: 16vh"
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
</style>
