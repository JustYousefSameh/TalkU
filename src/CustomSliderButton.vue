<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { info } from "@tauri-apps/plugin-log";
import { Mic, MicOff, Loader } from "lucide-vue-next";
import type { Status } from "./types";

const props = defineProps<{
    status: Status;
}>();

const emit = defineEmits<{
    (e: "update:status", value: Status): void;
    (e: "error", message: string): void;
}>();

const status = computed(() => props.status);
const hovered = ref(false);

const shockwave = ref(false);
const pop = ref(false);

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

type Action = "connect" | "disconnect" | null;

let lastAction: Action = null;

let busy = false;

// Successive failed connection attempts. Three in a row most likely means a
// stale talkuwg.conf, so on the third failure we force a fresh config from the
// server instead of just retrying the same one. Reset on any success.
let consecutiveFailures = 0;

async function reconcileStatus(): Promise<Status> {
    try {
        const line = await invoke<string>("get_vpn_status");
        if (line.startsWith("connected")) return "connected";
    } catch {
        // Status server unreachable -> genuinely disconnected.
    }
    return "disconnected";
}

async function disconnectCore() {
    try {
        await invoke("disconnect_vpn");
        emit("update:status", "disconnected");
    } catch (err) {
        emit("error", String(err));
        emit("update:status", "connected");
    }
}

async function disconnect() {
    if (busy) return;
    busy = true;
    try {
        await disconnectCore();
    } finally {
        busy = false;
    }
}

async function connect() {
    if (busy) return;
    busy = true;
    try {
        await invoke("check_config_and_connect");
        const result = await pollUntilConnected();
        consecutiveFailures = 0;
        emit("update:status", result);
    } catch (err) {
        if ((await reconcileStatus()) === "connected") {
            consecutiveFailures = 0;
            emit("update:status", "connected");
        } else {
            consecutiveFailures += 1;
            info(`consecutiveFailures=${consecutiveFailures}`);

            // Two strikes: likely a stale cached config. Delete it so the
            // next connect fetches a fresh one from the server.
            if (consecutiveFailures >= 2) {
                consecutiveFailures = 0;
                try {
                    await invoke("delete_config");
                } catch (retryErr) {
                    emit("error", String(retryErr));
                }
            } else {
                emit("error", String(err));
            }

            emit("update:status", "disconnected");
            // Remove the adapter + wstunnel left over from the failed attempt
            // (a stale config can leave the tunnel half-open). This calls the
            // unguarded core: `disconnect()` would be a no-op here because
            // we're still inside `connect()` and the `busy` guard is set.
            await disconnectCore();
        }
    } finally {
        busy = false;
    }
}

async function nextState() {
    if (busy || props.status === "connecting") return;
    if (props.status === "connected") {
        lastAction = "disconnect";
        emit("update:status", "disconnected");
        await disconnect();
    } else if (props.status === "disconnected") {
        lastAction = "connect";
        emit("update:status", "connecting");
        await connect();
    }
}

async function retryLastAction() {
    if (props.status === "connecting") return;
    if (lastAction === "disconnect") {
        emit("update:status", "disconnected");
        await disconnect();
    } else if (lastAction === "connect") {
        emit("update:status", "connecting");
        await connect();
    }
}

/// Runs the same connect/disconnect flow as a button click, but in a forced
/// direction. No-ops if the VPN is already in the requested state, so a manual
/// action (e.g. the user already connected) is not overridden. Used by the
/// auto-connect game watcher via a Tauri event.
async function autoAction(action: "connect" | "disconnect") {
    if (busy || props.status === "connecting") return;
    if (action === "connect" && props.status === "connected") return; // already connected
    if (action === "disconnect" && props.status === "disconnected") return; // already disconnected
    lastAction = action;
    emit(
        "update:status",
        action === "disconnect" ? "disconnected" : "connecting",
    );
    if (action === "disconnect") await disconnect();
    else await connect();
}

defineExpose({ retryLastAction, autoAction });

async function pollUntilConnected(): Promise<Status> {
    const deadline = Date.now() + 5000;
    const startTime = Date.now();
    while (Date.now() < deadline) {
        try {
            const line = await invoke<string>("get_vpn_status");
            const timeTaken = Date.now() - startTime;
            info(`pollUntilConnected: timeTaken=${timeTaken / 1000}`);
            if (line.startsWith("connected")) return "connected";
        } catch {
            // Status server not reachable yet; keep polling.
        }
        await new Promise((r) => setTimeout(r, 100));
    }
    throw new Error("Failed to connect.");
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
    animation: toggle-pop 0.3s ease-out;
}

@keyframes toggle-pop {
    0% {
        transform: scale(1);
    }
    40% {
        transform: scale(1.06);
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
