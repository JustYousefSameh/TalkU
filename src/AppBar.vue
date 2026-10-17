<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { Settings, Minus, X, Download } from "lucide-vue-next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { info } from "@tauri-apps/plugin-log";
import SettingsMenu from "./SettingsMenu.vue";
import MonitorMenu from "./MonitorMenu.vue";
import { useUnreachableScan } from "./composables/useUnreachableScan";
import { useUpdateCheck } from "./composables/useUpdateCheck";
import UpdateDialog from "./UpdateDialog.vue";

const settingsOpen = ref(false);
const monitorOpen = ref(false);
const appbarRef = ref<HTMLDivElement | null>(null);

const { scanning, start: startScan, dispose } = useUnreachableScan();
const {
    newVersion,
    step: updateStep,
    progress,
    speed,
    error: updateError,
    promptUpdate,
    closeDialog,
    install,
} = useUpdateCheck();

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
    dispose();
});
</script>

<template>
    <main ref="appbarRef" class="appbar-root">
        <div
            class="flex flex-row justify-between items-center w-full pt-2 pl-3 pr-4 appbar-slide"
        >
            <div class="flex-1 flex flex-row items-baseline">
                <span class="app-title text-gray-300 pr-1 text-sm">TalkU</span>
                <span class="text-gray-500 text-xs">V3.0</span>
                <!-- Small green "new version" pill, only shown when GitHub
                     has a release newer than this build. -->
                <button
                    v-if="newVersion"
                    class="update-btn"
                    type="button"
                    :title="`Update to TalkU V${newVersion}`"
                    @click="promptUpdate"
                >
                    <Download class="h-3" />
                    V{{ newVersion }}
                </button>
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
        <MonitorMenu
            :open="monitorOpen"
            :scanning="scanning"
            @close="monitorOpen = false"
            @start-scan="startScan"
        />
        <UpdateDialog
            v-if="newVersion"
            :step="updateStep"
            :version="newVersion"
            :progress="progress"
            :speed="speed"
            :error="updateError"
            @update="install"
            @close="closeDialog"
        />
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

.update-btn {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    margin-left: 8px;
    padding: 2px 8px 2px 4px;
    border: 1px solid rgba(35, 164, 70, 0.55);
    border-radius: 999px;
    background: rgba(35, 164, 70, 0.16);
    color: #7ee787;
    font-family: "Roboto", sans-serif;
    font-size: 11px;
    line-height: 1.4;
    cursor: pointer;
    transition:
        background 0.2s ease,
        border-color 0.2s ease;
}

.update-btn:hover {
    background: rgba(35, 164, 70, 0.28);
    border-color: #23a446;
}
</style>
