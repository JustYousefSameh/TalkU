<script setup lang="ts">
import { ref, watch, computed, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Monitor, Search, X, RefreshCw } from "lucide-vue-next";
import ErrorBox from "./ErrorBox.vue";

const props = defineProps<{
    open: boolean;
}>();

defineEmits<{
    close: [];
}>();

interface ProcessInfo {
    pid: number;
    name: string;
    count: number;
    cpu_percent: number;
    memory_kb: number;
}

const processes = ref<ProcessInfo[]>([]);
const search = ref("");
const loading = ref(false);
const error = ref("");
const scanning = ref<string | null>(null);
const popupMessage = ref<string | null>(null);
const popupVariant = ref<"error" | "success">("success");
let pollTimer: ReturnType<typeof setInterval> | null = null;

const filteredProcesses = computed(() => {
    const q = search.value.trim().toLowerCase();
    if (!q) return processes.value;
    return processes.value.filter((p) => p.name.toLowerCase().includes(q));
});

async function refresh() {
    loading.value = true;
    error.value = "";
    try {
        processes.value = await invoke<ProcessInfo[]>("list_processes");
    } catch (e) {
        error.value = String(e);
    } finally {
        loading.value = false;
    }
}

async function scan(name: string) {
    if (scanning.value) return; // only one in-flight at a time
    scanning.value = name;
    try {
        await invoke("collect_unreachable", { processName: name, seconds: 15 });
        popupVariant.value = "success";
        popupMessage.value = `Scan initiated for ${name}. Please reproduce the issue so we can capture the logs.`;
    } catch (e) {
        popupVariant.value = "error";
        popupMessage.value = String(e);
    } finally {
        scanning.value = null;
    }
}

watch(
    () => props.open,
    (open) => {
        if (open) {
            refresh();
            pollTimer = setInterval(refresh, 4000);
        } else if (pollTimer) {
            clearInterval(pollTimer);
            pollTimer = null;
        }
        if (!open) {
            popupMessage.value = null;
            scanning.value = null;
            search.value = "";
        }
    },
);

onUnmounted(() => {
    if (pollTimer) clearInterval(pollTimer);
});
</script>

<template>
    <Transition name="menu-fade">
        <div
            v-if="open"
            class="monitor-menu"
            data-tauri-drag-region="false"
            @click.stop
        >
            <div class="menu-head">
                <Monitor class="menu-head-icon" />
                <span>Monitor</span>
                <span class="menu-count"
                    >{{ filteredProcesses.length }} running</span
                >
                <button
                    class="menu-head-action"
                    type="button"
                    title="Refresh"
                    :disabled="loading"
                    @click="refresh"
                >
                    <RefreshCw class="h-4" :class="{ spin: loading }" />
                </button>
                <button
                    class="menu-head-action"
                    type="button"
                    title="Close"
                    @click="$emit('close')"
                >
                    <X class="h-4" />
                </button>
            </div>
            <div class="menu-body">
                <div class="search-box">
                    <Search class="search-icon" />
                    <input
                        v-model="search"
                        type="text"
                        class="search-input"
                        placeholder="Search processes…"
                        spellcheck="false"
                    />
                    <button
                        v-if="search"
                        type="button"
                        class="search-clear"
                        title="Clear search"
                        @click="search = ''"
                    >
                        <X class="h-4" />
                    </button>
                </div>
                <div
                    v-if="loading && processes.length === 0"
                    class="procs-empty"
                >
                    Loading processes…
                </div>
                <div v-else-if="error" class="procs-empty">{{ error }}</div>
                <div
                    v-else-if="filteredProcesses.length === 0"
                    class="procs-empty"
                >
                    {{
                        search.trim()
                            ? `No processes match "${search.trim()}"`
                            : "No processes found"
                    }}
                </div>
                <div v-else class="procs-list">
                    <div
                        v-for="p in filteredProcesses"
                        :key="p.name"
                        class="proc-row"
                        :class="{ disabled: scanning !== null }"
                        role="button"
                        tabindex="0"
                        :title="'Run unreachable scan on ' + p.name"
                        @click="scan(p.name)"
                        @keydown.enter="scan(p.name)"
                    >
                        <span class="proc-name">{{ p.name }}</span>
                        <span class="proc-pid">{{ p.pid }}</span>
                        <span v-if="scanning === p.name" class="scan-ind"
                            >scanning…</span
                        >
                    </div>
                </div>
            </div>
            <ErrorBox
                :open="popupMessage !== null"
                :message="popupMessage || ''"
                :variant="popupVariant"
                :show-cancel="false"
                confirm-text="OK"
                @confirm="popupMessage = null"
                @cancel="popupMessage = null"
            />
        </div>
    </Transition>
</template>

<style scoped>
.monitor-menu {
    font-family: "Roboto", sans-serif;
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -47%);
    display: flex;
    flex-direction: column;
    width: 280px;
    max-height: calc(100vh - 48px);
    z-index: 1001;
    background: rgba(12, 14, 16, 0.92);
    backdrop-filter: blur(14px);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    box-shadow:
        0 18px 40px rgba(0, 0, 0, 0.55),
        0 2px 8px rgba(0, 0, 0, 0.4);
    overflow: hidden;
    user-select: none;
}

.menu-head {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 10px 9px 14px;
    color: #e5e7eb;
    font-size: 12.5px;
    font-weight: 600;
    border-bottom: 1px solid rgba(255, 255, 255, 0.07);
    background: rgba(255, 255, 255, 0.03);
}

.menu-head-icon {
    color: #23a446;
    width: 15px;
    height: 15px;
}

.menu-count {
    font-size: 10px;
    font-weight: 400;
    color: #8b919a;
    flex: 1;
}

.menu-head-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border: none;
    background: transparent;
    color: #6b7280;
    border-radius: 5px;
    cursor: pointer;
    transition:
        color 0.14s ease,
        background-color 0.14s ease;
    flex-shrink: 0;
}

.menu-head-action:hover {
    color: #e5e7eb;
    background: rgba(255, 255, 255, 0.08);
}

.menu-head-action:disabled {
    opacity: 0.5;
    cursor: default;
}

.menu-body {
    padding: 6px;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    scrollbar-color: rgba(35, 164, 70, 0.5) transparent;
}

.menu-body::-webkit-scrollbar {
    width: 6px;
}

.menu-body::-webkit-scrollbar-thumb {
    background: rgba(35, 164, 70, 0.5);
    border-radius: 9999px;
}

.menu-body::-webkit-scrollbar-track {
    background: transparent;
}

.search-box {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 6px;
    padding: 0 8px;
    height: 30px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.03);
}

.search-icon {
    width: 13px;
    height: 13px;
    color: #6b7280;
    flex-shrink: 0;
}

.search-input {
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    outline: none;
    color: #e5e7eb;
    font-family: "Roboto", sans-serif;
    font-size: 11.5px;
}

.search-input::placeholder {
    color: #6b7280;
}

.search-clear {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border: none;
    background: transparent;
    color: #6b7280;
    border-radius: 4px;
    cursor: pointer;
    flex-shrink: 0;
    padding: 0;
}

.search-clear:hover {
    color: #e5e7eb;
    background: rgba(255, 255, 255, 0.08);
}

.procs-list {
    display: flex;
    flex-direction: column;
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.02);
}

.proc-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    font-size: 11.5px;
    color: #e5e7eb;
    cursor: pointer;
    transition:
        background-color 0.12s ease,
        color 0.12s ease;
}

.proc-row:hover {
    background: rgba(35, 164, 70, 0.12);
    color: #ffffff;
}

.proc-row:focus-visible {
    outline: 1px solid rgba(35, 164, 70, 0.6);
    outline-offset: -1px;
}

.proc-row + .proc-row {
    border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.proc-name {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.proc-pid {
    font-size: 10px;
    color: #8b919a;
    white-space: nowrap;
}

.scan-ind {
    font-size: 10px;
    color: #23a446;
    white-space: nowrap;
}

.proc-row.disabled {
    pointer-events: none;
    opacity: 0.55;
}

.procs-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px 12px;
    color: #8b919a;
    font-size: 11.5px;
}

.spin {
    animation: spin 0.8s linear infinite;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

.menu-fade-enter-active,
.menu-fade-leave-active {
    transition:
        opacity 0.22s ease,
        transform 0.22s ease;
}

.menu-fade-enter-from,
.menu-fade-leave-to {
    opacity: 0;
    transform: translate(-50%, -47%) translateY(18px);
}
</style>
