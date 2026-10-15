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

type ServerStatus = "checking" | "online" | "unavailable";
const serverStatus = ref<ServerStatus>("checking");

let interval: number | undefined;
let bonusTimer: number | undefined;
let serverStatusInterval: number | undefined;

function applyBonus(delta: number) {
    fakeBonus.value += delta;

    if (bonusTimer !== undefined) {
        window.clearTimeout(bonusTimer);
    }

    bonusTimer = window.setTimeout(() => {
        fakeBonus.value = 0;
        bonusTimer = undefined;
    }, 120000);
}

watch(
    () => props.status,
    (next, prev) => {
        if (next === "connected" && prev !== "connected") {
            applyBonus(1);
        } else if (next === "disconnected" && prev !== "disconnected") {
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

async function refreshServerStatus() {
    try {
        const result = await invoke<string>(
            "get_community_fund_server_status",
        );

        serverStatus.value = result === "online" ? "online" : "unavailable";
    } catch (err) {
        console.error("Failed to check Community Fund server status:", err);
        serverStatus.value = "unavailable";
    }
}

onMounted(() => {
    refresh();
    interval = window.setInterval(refresh, 120000);

    refreshServerStatus();
    serverStatusInterval = window.setInterval(refreshServerStatus, 30000);
});

onUnmounted(() => {
    if (interval !== undefined) {
        window.clearInterval(interval);
    }

    if (bonusTimer !== undefined) {
        window.clearTimeout(bonusTimer);
    }

    if (serverStatusInterval !== undefined) {
        window.clearInterval(serverStatusInterval);
    }
});
</script>

<template>
    <main
        class="flex flex-row fixed bottom-0 right-0 gap-3 justify-center items-center p-3 users-slide"
    >
        <div
            class="status-group"
            :title="
                serverStatus === 'online'
                    ? 'Community Fund API is reachable'
                    : serverStatus === 'checking'
                      ? 'Checking Community Fund API'
                      : 'Community Fund API is unavailable'
            "
        >
            <span
                class="status-dot"
                :class="`status-dot--${serverStatus}`"
            ></span>
            <span class="status-label">Server Status</span>
        </div>

        <div class="users-group" title="Currently connected users">
            <span class="dot"></span>
            <span class="text-gray-400 text-xs leading-none users-label">
                <NumberFlow
                    :value="shown"
                    :format="{ minimumIntegerDigits: 1 }"
                    class="numflow"
                />
                Users Connected
            </span>
        </div>
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

.status-group,
.users-group {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 5px;
    white-space: nowrap;
}

.status-label {
    color: rgba(209, 213, 219, 0.9);
    font-size: 11px;
    line-height: 1;
}

.status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    display: inline-block;
    flex-shrink: 0;
}

.status-dot--checking {
    background-color: #9ca3af;
    box-shadow: 0 0 5px rgba(156, 163, 175, 0.4);
}

.status-dot--online {
    background-color: #4ade80;
    box-shadow: 0 0 6px 1px rgba(74, 222, 128, 0.4);
    animation: pulse 3s ease-in-out infinite;
}

.status-dot--unavailable {
    background-color: #f87171;
    box-shadow: 0 0 6px 1px rgba(248, 113, 113, 0.4);
}

.dot {
    width: 8px;
    height: 8px;
    background-color: rgba(74, 222, 128, 0.8);
    border-radius: 50%;
    display: inline-block;
    flex-shrink: 0;
    animation: pulse 3s ease-in-out infinite;
    box-shadow: 0 0 6px 1px rgba(74, 222, 128, 0.35);
}

.users-label {
    display: inline-flex;
    align-items: center;
    gap: 3px;
}

.numflow {
    display: inline-block;
}

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
