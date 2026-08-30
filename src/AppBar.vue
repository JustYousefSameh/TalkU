<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { Settings, Minus, X } from "lucide-vue-next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { info } from "@tauri-apps/plugin-log";
import SettingsMenu from "./SettingsMenu.vue";
import MonitorMenu from "./MonitorMenu.vue";

const settingsOpen = ref(false);
const monitorOpen = ref(false);
const appbarRef = ref<HTMLDivElement | null>(null);

// Minimize to tray when the close button is pressed
async function handleClose() {
    info("Minimizing to tray");
    await getCurrentWindow().hide();
}

async function minimizeWindow() {
    await getCurrentWindow().minimize();
}

function toggleSettings() {
    monitorOpen.value = false;
    settingsOpen.value = !settingsOpen.value;
}

function openMonitor() {
    settingsOpen.value = false;
    monitorOpen.value = true;
}

function onDocumentClick(e: MouseEvent) {
    if (appbarRef.value && !appbarRef.value.contains(e.target as Node)) {
        settingsOpen.value = false;
        monitorOpen.value = false;
    }
}

onMounted(() => {
    document.addEventListener("click", onDocumentClick);
});
onUnmounted(() => {
    document.removeEventListener("click", onDocumentClick);
});
</script>

<template>
    <main ref="appbarRef" class="appbar-root">
        <div
            class="flex flex-row justify-between items-center w-full pt-2 pl-3 pr-4 appbar-slide"
        >
            <div class="flex-1">
                <span class="app-title text-gray-300 pr-1 text-sm">TalkU</span>
                <span class="text-gray-500 text-xs">V2.4</span>
            </div>
            <div class="flex flex-row gap-5 items-center relative">
                <button
                    class="text-gray-300 hover:text-white cursor-pointer transition-colors duration-150 settings-btn"
                    @click.stop="toggleSettings"
                    :class="{ active: settingsOpen }"
                >
                    <Settings class="h-5" />
                </button>

                <button
                    class="text-gray-300 hover:text-white cursor-pointer transition-colors duration-150"
                    @click="minimizeWindow"
                >
                    <Minus class="h-5" />
                </button>
                <button
                    class="text-gray-300 hover:text-white cursor-pointer transition-colors duration-150"
                    @click="handleClose"
                >
                    <X class="h-5" />
                </button>
            </div>
        </div>

        <SettingsMenu :open="settingsOpen" @open-monitor="openMonitor" />
        <MonitorMenu :open="monitorOpen" @close="monitorOpen = false" />
    </main>
</template>

<style scoped>
@font-face {
    font-family: "Roboto";
    src: url("/Roboto-Regular.ttf") format("truetype");
    font-weight: 400;
    font-style: normal;
}

.app-title {
    font-family: "Roboto", sans-serif;
}

.appbar-slide {
    animation: slide-down 0.6s ease-out both;
}

@keyframes slide-down {
    from {
        transform: translateY(-100%);
        opacity: 0;
    }
    to {
        transform: translateY(0);
        opacity: 1;
    }
}

/* Keep the whole app bar (and its settings dropdown) above all other UI. */
.appbar-root {
    position: relative;
    z-index: 999;
}

.settings-btn {
    transform: rotate(0deg);
    transition:
        color 0.15s ease,
        transform 0.3s ease;
}

.settings-btn.active {
    color: #23a446;
    transform: rotate(45deg);
}
</style>
