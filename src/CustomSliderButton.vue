<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Mic, MicOff, Loader } from "lucide-vue-next";
import type { Status } from "./types";

const props = defineProps<{
    status: Status;
}>();

const emit = defineEmits<{
    (e: "update:status", value: Status): void;
}>();

const status = computed(() => props.status);
const hovered = ref(false);

const shockwave = ref(false);
const pop = ref(false);
const buttonRef = ref<HTMLButtonElement | null>(null);

let shockwaveTimer: number | undefined;

watch(
    () => props.status,
    (s) => {
        if (s === "connected") {
            shockwave.value = false;
            pop.value = false;
            if (shockwaveTimer !== undefined)
                window.clearTimeout(shockwaveTimer);
            shockwaveTimer = window.setTimeout(() => {
                shockwave.value = true;
                pop.value = true;
            }, 300);
        }
    },
);

async function nextState() {
    if (props.status === "connected") {
        emit("update:status", "connecting");
        try {
            await invoke("disconnect_vpn");
            emit("update:status", "disconnected");
        } catch (err) {
            console.error(err);
            emit("update:status", "connected");
        }
    } else if (props.status === "disconnected") {
        emit("update:status", "connecting");
        try {
            await invoke("connect_vpn");
            const result = await pollUntilConnected();
            emit("update:status", result);
        } catch (err) {
            console.error(err);
            emit("update:status", "disconnected");
        }
    }
}

async function pollUntilConnected(): Promise<Status> {
    const deadline = Date.now() + 30000;
    while (Date.now() < deadline) {
        try {
            const line = await invoke<string>("get_vpn_status");
            if (line.startsWith("connected")) return "connected";
        } catch {
            // Status server not reachable yet; keep polling.
        }
        await new Promise((r) => setTimeout(r, 1000));
    }
    return "disconnected";
}
</script>

<template>
    <main
        class="absolute inset-x-0 flex flex-col items-center gap-20 pointer-events-none status-slide"
        style="top: 25vh"
    >
        <div v-if="shockwave" class="shockwave" aria-hidden="true"></div>
        <Transition name="text-swap" mode="out-in">
            <span
                :key="status"
                :class="['status-text', 'status-text--' + status]"
            >
                <template v-if="status === 'connected'">Connected</template>
                <template v-else-if="status === 'connecting'"
                    >Connecting...</template
                >
                <template v-else>Disconnected</template>
            </span>
        </Transition>
        <button
            ref="buttonRef"
            type="button"
            :class="[
                'toggle',
                'toggle--' + status,
                { hovered: hovered, pop: pop },
            ]"
            @click="nextState"
            @mouseenter="hovered = true"
            @mouseleave="hovered = false"
        >
            <div class="toggle-track">
                <div class="toggle-thumb">
                    <MicOff
                        v-if="status === 'disconnected'"
                        key="micoff"
                        class="thumb-icon"
                    />
                    <Loader
                        v-else-if="status === 'connecting'"
                        key="loader"
                        class="thumb-icon spin"
                    />
                    <Mic v-else key="mic" class="thumb-icon" />
                </div>
            </div>
        </button>
    </main>
</template>

<style scoped>
@font-face {
    font-family: "Product Sans";
    src: url("/Product Sans Bold.ttf") format("truetype");
    font-weight: bold;
    font-style: normal;
}

.status-slide {
    animation: status-up 0.7s ease-out both;
}

.shockwave {
    position: fixed;
    bottom: 0px;
    width: 180px;
    height: 80px;
    border: 3px solid rgba(35, 164, 70, 0.5);
    border-radius: 9999px;
    box-shadow: 0 0 14px 3px rgba(35, 164, 70, 0.35);
    animation: shockwave-expand 1s ease-out forwards;
    pointer-events: none;
}

@keyframes shockwave-expand {
    0% {
        transform: scale(1);
        opacity: 0.55;
    }
    100% {
        transform: scale(5);
        opacity: 0;
    }
}

.status-text {
    font-family: "Product Sans", sans-serif;
    font-size: 3rem;
    text-shadow: 3px 5px 8px rgba(0, 0, 0, 0.3);
}

.status-text--disconnected {
    color: #b81a15;
}

.status-text--connecting {
    color: #b8aa15;
}

.status-text--connected {
    color: #15b833;
}

.text-swap-enter-active {
    transition: all 0.3s ease;
}

.text-swap-leave-active {
    transition: all 0.2s ease;
}

.text-swap-leave-to {
    opacity: 0;
    transform: translateY(-12px);
}

.text-swap-enter-from {
    opacity: 0;
    transform: translateY(12px);
}

@keyframes status-up {
    from {
        opacity: 0;
        transform: translateY(24px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.toggle {
    pointer-events: auto;
    border: none;
    background: none;
    padding: 0;
    cursor: pointer;
    transform: scale(1);
    transition: transform 0.15s ease;
    outline: none;
}

.toggle.hovered {
    transform: scale(1.07);
}

.toggle:active {
    transform: scale(1.02);
}

.toggle.pop {
    animation: toggle-pop 0.25s ease-out;
}

@keyframes toggle-pop {
    0% {
        transform: scale(1);
    }
    40% {
        transform: scale(1.15);
    }
    100% {
        transform: scale(1);
    }
}

.toggle-track {
    display: flex;
    align-items: center;
    width: 180px;
    height: 80px;
    padding: 7px;
    border-radius: 9999px;
    box-sizing: border-box;
    transition:
        width 0.35s ease,
        background-color 0.3s ease,
        box-shadow 0.3s ease;
    box-shadow: 3px 5px 10px rgba(0, 0, 0, 0.25);
}

.toggle--disconnected .toggle-track {
    width: 180px;
    background-color: #3b0b0b;
}

.toggle--connecting .toggle-track {
    width: 80px;
    background-color: #26260a;
}

.toggle--connected .toggle-track {
    width: 180px;
    background-color: #0c2810;
}

.toggle-thumb {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 66px;
    height: 66px;
    border-radius: 9999px;
    flex-shrink: 0;
    transform: translateX(0);
    transition:
        transform 0.35s cubic-bezier(0.4, 0, 0.2, 1),
        background-color 0.3s ease;
}

.toggle--disconnected .toggle-thumb {
    transform: translateX(0);
    background-color: #531313;
}

.toggle--connecting .toggle-thumb {
    transform: translateX(0px);
    background-color: #3c3d11;
}

.toggle--connected .toggle-thumb {
    transform: translateX(100px);
    background-color: #143c1c;
}

.icon-swap-enter-active {
    transition:
        opacity 0.1s ease,
        transform 0.1s ease;
}

.icon-swap-leave-active {
    transition:
        opacity 0.1s ease,
        transform 0.1s ease;
}

.icon-swap-enter-from {
    opacity: 0;
    transform: scale(0.6);
}

.icon-swap-leave-to {
    opacity: 0;
    transform: scale(0.6);
}

.thumb-icon {
    width: 46px;
    height: 46px;
}

.toggle--disconnected .thumb-icon {
    color: #ed4444;
}

.toggle--connecting .thumb-icon {
    color: #d5db27;
}

.toggle--connected .thumb-icon {
    color: #23a446;
}

.spin {
    animation: spin 1.75s linear infinite;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}
</style>
