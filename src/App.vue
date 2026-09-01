<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import AppBar from "./AppBar.vue";
import CustomSliderButton from "./CustomSliderButton.vue";
import SocialLink from "./SocialLink.vue";
import ActiveUsers from "./ActiveUsers.vue";
import GradientBackgroundWithImage from "./GradientBackgroundWithImage.vue";
import ConnectionTimer from "./ConnectionTimer.vue";
import ErrorBox from "./ErrorBox.vue";
import { useMessageBox } from "./stores/messageBox";
import { useAudioCues } from "./stores/audioCues";
import connectSound from "./assets/talku_connected.wav";
import disconnectSound from "./assets/talku_disconnected.wav";
import type { Status } from "./types";

const status = ref<Status>("disconnected");
const sliderRef = ref<InstanceType<typeof CustomSliderButton> | null>(null);
const { enabled: audioCues, load: loadAudioCues } = useAudioCues();
const {
    message: boxMessage,
    title: boxTitle,
    variant: boxVariant,
    confirmText: boxConfirmText,
    showCancel: boxShowCancel,
    show: boxShow,
    hide: boxHide,
    confirm: boxConfirm,
} = useMessageBox();

function onSliderError(message: string) {
    boxShow({
        message,
        title: "Connection failed",
        variant: "error",
        confirmText: "Retry",
        showCancel: true,
        onConfirm: () => sliderRef.value?.retryLastAction(),
    });
}

let audio: HTMLAudioElement | null = null;

function playSound(src: string) {
    try {
        if (audio) {
            audio.pause();
            audio.currentTime = 0;
        }
        audio = new Audio(src);
        audio.play().catch(() => {});
    } catch {}
}

watch(status, (next, prev) => {
    if (prev === next) return;
    if (!audioCues.value) return;
    if (next === "connected") playSound(connectSound);
    else if (next === "disconnected") playSound(disconnectSound);
});

// Mirror the VPN state into the tray icon: green logo while connected, red
// logo while disconnected.
async function syncTray(connected: boolean) {
    try {
        await invoke("set_tray_icon", { connected });
    } catch (e) {
        console.error("failed to set tray icon", e);
    }
}

watch(status, (next) => syncTray(next === "connected"));

// Auto-connect on game launch: the Rust game watcher emits events on the
// rising/falling edge of a monitored game process, and we run the exact same
// connect/disconnect flow as a button click so the UI updates.
let unlistenConnect: (() => void) | null = null;
let unlistenDisconnect: (() => void) | null = null;

// Suppress the default webview context menu (copy/save image, etc.).
const suppressContextMenu = (e: MouseEvent) => e.preventDefault();

onMounted(async () => {
    window.addEventListener("contextmenu", suppressContextMenu);
    loadAudioCues();
    syncTray(status.value === "connected");
    unlistenConnect = await listen("game-connect", () => {
        sliderRef.value?.autoAction("connect");
    });
    unlistenDisconnect = await listen("game-disconnect", () => {
        sliderRef.value?.autoAction("disconnect");
    });
});

onUnmounted(() => {
    window.removeEventListener("contextmenu", suppressContextMenu);
    unlistenConnect?.();
    unlistenDisconnect?.();
});
</script>

<template>
    <main>
        <div class="app-main" data-tauri-drag-region="deep">
            <GradientBackgroundWithImage :status="status" />
            <AppBar />
            <CustomSliderButton
                ref="sliderRef"
                v-model:status="status"
                @error="onSliderError"
            />
            <ConnectionTimer :status="status" />
            <ActiveUsers :status="status" />
            <ErrorBox
                :open="boxMessage !== null"
                :title="boxTitle"
                :message="boxMessage || ''"
                :variant="boxVariant"
                :confirm-text="boxConfirmText"
                :show-cancel="boxShowCancel"
                @confirm="boxConfirm()"
                @cancel="boxHide()"
            />
            <div class="socials">
                <SocialLink
                    image="/discord.png"
                    alt="Discord"
                    link="https://discord.gg/mph7jETDv9"
                />
                <SocialLink
                    image="/github.png"
                    alt="GitHub"
                    link="https://github.com/JustYousefSameh/TalkU"
                />
            </div>
        </div>
    </main>
</template>

<style>
html,
body {
    margin: 0;
    overflow: hidden;
    height: 100%;
    background: transparent;
    user-select: none;
    -webkit-user-select: none;
}

:focus {
    outline: none;
}

:root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
}

.app-main {
    overflow: hidden;
    width: 100vw;
    height: 100vh;
    border-radius: 24px;
    background: transparent;
}

.socials {
    position: absolute;
    left: 16px;
    bottom: 16px;
    display: flex;
    gap: 12px;
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
</style>
