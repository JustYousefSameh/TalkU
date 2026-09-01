<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NumberFlow from "@number-flow/vue";
import type { Status } from "./types";

const props = defineProps<{
    status: Status;
}>();

const baseCount = ref<number>(0);
const fakeBonus = ref<number>(0);
const shown = computed(() => baseCount.value + fakeBonus.value);

let interval: number | undefined;
let bonusTimer: number | undefined;

function applyBonus(delta: number) {
    fakeBonus.value += delta;
    if (bonusTimer !== undefined) window.clearTimeout(bonusTimer);
    bonusTimer = window.setTimeout(() => {
        fakeBonus.value = 0;
        bonusTimer = undefined;
    }, 120000);
}

watch(
    () => props.status,
    (next, prev) => {
        if (next === "connected" && prev !== "connected") {
            // Just connected -> fake +1 as instant feedback, removed after 2 min.
            applyBonus(1);
        } else if (next === "disconnected" && prev !== "disconnected") {
            // Just disconnected -> fake -1 as instant feedback, removed after 2 min.
            applyBonus(-1);
        }
    },
);

async function refresh() {
    try {
        baseCount.value = await invoke<number>("get_connected_users_count");
    } catch (err) {
        console.error(err);
    }
}

onMounted(() => {
    refresh();
    interval = window.setInterval(refresh, 120000);
});

onUnmounted(() => {
    if (interval !== undefined) window.clearInterval(interval);
    if (bonusTimer !== undefined) window.clearTimeout(bonusTimer);
});
</script>

<template>
    <main
        class="flex flex-row fixed bottom-0 right-0 gap-1 justify-center items-center p-4 users-slide"
    >
        <div class="dot"></div>
        <span class="text-gray-400 text-xs overflow-hidden leading-none">
            <NumberFlow
                :value="shown"
                :format="{ minimumIntegerDigits: 1 }"
                class="numflow"
            />
        </span>
    </main>
</template>

<style scoped>
.users-slide {
    animation: slide-up 0.6s ease-out both;
}

@keyframes slide-up {
    from {
        transform: translateY(30px);
        opacity: 0;
    }
    to {
        transform: translateY(0);
        opacity: 1;
    }
}

.dot {
    width: 8px;
    height: 8px;
    background-color: rgba(74, 222, 128, 0.8);
    border-radius: 50%;
    display: inline-block;
    animation: pulse 3s ease-in-out infinite;
    box-shadow: 0 0 6px 1px rgba(74, 222, 128, 0.35);
}

.numflow {
    display: inline-block;
}

/*Smooth glowing pulse animation*/
@keyframes pulse {
    0%,
    100% {
        opacity: 0.5;
        box-shadow: 0 0 3px 0 rgba(74, 222, 128, 0.25);
    }
    50% {
        opacity: 0.9;
        box-shadow: 0 0 9px 2px rgba(74, 222, 128, 0.5);
    }
}
</style>
