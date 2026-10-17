<script setup lang="ts">
import { computed } from "vue";
import { ArrowDown } from "lucide-vue-next";
import type { DownloadProgress, UpdateStep } from "./composables/useUpdateCheck";

const props = defineProps<{
    step: UpdateStep | null;
    version: string;
    progress: DownloadProgress;
    /** Bytes per second. */
    speed: number;
    error: string;
}>();

const emit = defineEmits<{
    (e: "update"): void;
    (e: "close"): void;
}>();

const percent = computed(() =>
    props.progress.total > 0
        ? Math.min(100, Math.floor((props.progress.received / props.progress.total) * 100))
        : 0,
);

function megabytes(bytes: number): string {
    return (bytes / 1024 / 1024).toFixed(1);
}

// The download can't be stopped halfway, so only let people click away from
// the prompt and the error screen.
function onOverlayClick() {
    if (props.step !== "downloading") emit("close");
}
</script>

<template>
    <!-- Same glass card and buttons as ErrorBox, so it feels like part of the app. -->
    <Teleport to="body">
        <Transition name="popup">
            <div v-if="step" class="popup-overlay" @click.self="onOverlayClick">
                <div class="popup-box">
                    <div
                        class="popup-icon"
                        :class="{ failed: step === 'failed' }"
                        aria-hidden="true"
                    >
                        <span v-if="step === 'failed'">!</span>
                        <ArrowDown v-else class="h-7 w-7" :stroke-width="3" />
                    </div>

                    <template v-if="step === 'prompt'">
                        <p class="popup-title">Update available</p>
                        <p class="popup-message">
                            TalkU <span class="version">V{{ version }}</span> is
                            ready to install.
                        </p>
                        <p class="popup-note">
                            TalkU will close while the update installs.
                        </p>
                        <div class="popup-actions">
                            <button type="button" class="btn btn-cancel" @click="emit('close')">
                                Later
                            </button>
                            <button type="button" class="btn btn-confirm" @click="emit('update')">
                                Update
                            </button>
                        </div>
                    </template>

                    <template v-else-if="step === 'downloading'">
                        <p class="popup-title">
                            Downloading <span class="version">V{{ version }}</span>
                        </p>
                        <div class="bar">
                            <div class="bar-fill" :style="{ width: `${percent}%` }"></div>
                        </div>
                        <div class="stats">
                            <span>
                                {{ megabytes(progress.received) }} /
                                {{ progress.total ? megabytes(progress.total) : "…" }} MB
                            </span>
                            <span>{{ megabytes(speed) }} MB/s</span>
                            <span class="version">{{ percent }}%</span>
                        </div>
                        <p class="popup-note">
                            The installer opens as soon as the download finishes.
                        </p>
                    </template>

                    <template v-else>
                        <p class="popup-title">Update failed</p>
                        <p class="popup-message">{{ error }}</p>
                        <div class="popup-actions">
                            <button type="button" class="btn btn-cancel" @click="emit('close')">
                                Close
                            </button>
                            <button type="button" class="btn btn-confirm" @click="emit('update')">
                                Retry
                            </button>
                        </div>
                    </template>
                </div>
            </div>
        </Transition>
    </Teleport>
</template>

<style scoped>
.popup-overlay {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.55);
    border-radius: 16px;
    z-index: 2000;
}

.popup-box {
    width: 300px;
    max-width: 84vw;
    padding: 28px 24px 20px;
    border-radius: 24px;
    background: rgba(26, 26, 30, 0.35);
    backdrop-filter: blur(16px);
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow:
        0 12px 40px rgba(0, 0, 0, 0.6),
        inset 0 1px 0 rgba(255, 255, 255, 0.06);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    font-family: "Roboto", sans-serif;
}

.popup-icon {
    width: 52px;
    height: 52px;
    border-radius: 50%;
    background: radial-gradient(circle at 30% 25%, #2aa84f 0%, #176037 70%);
    box-shadow: 0 4px 18px rgba(35, 164, 70, 0.45);
    color: #fff;
    font-family: "Product Sans", "Roboto", sans-serif;
    font-size: 30px;
    font-weight: 700;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
}

.popup-icon.failed {
    background: radial-gradient(circle at 30% 25%, #ed4444 0%, #b81a15 70%);
    box-shadow: 0 4px 18px rgba(184, 26, 21, 0.45);
}

.popup-title {
    margin: 0;
    color: #f4f4f8;
    font-size: 16px;
    font-weight: 700;
    text-align: center;
}

.popup-message {
    margin: 0;
    color: #d6d6db;
    font-size: 14px;
    line-height: 1.5;
    text-align: center;
    word-break: break-word;
    max-height: 104px;
    overflow-y: auto;
}

/* Secondary info, kept quiet so the main message stands out. */
.popup-note {
    margin: 0;
    color: #8b919a;
    font-size: 12px;
    line-height: 1.5;
    text-align: center;
}

.version {
    color: #4ade80;
    font-weight: 700;
}

.bar {
    width: 100%;
    height: 8px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    overflow: hidden;
}

.bar-fill {
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(90deg, #1d7a38 0%, #23a446 100%);
    box-shadow: 0 0 10px rgba(35, 164, 70, 0.5);
    transition: width 0.2s ease;
}

.stats {
    width: 100%;
    display: flex;
    justify-content: space-between;
    color: #8b919a;
    font-size: 12px;
}

.popup-actions {
    display: flex;
    gap: 12px;
    width: 100%;
    margin-top: 4px;
}

.btn {
    flex: 1;
    padding: 10px 0;
    border: none;
    border-radius: 12px;
    font-family: "Roboto", sans-serif;
    font-size: 14px;
    font-weight: 600;
    letter-spacing: 0.2px;
    cursor: pointer;
    transition:
        transform 0.1s ease,
        background 0.2s ease;
}

.btn:active {
    transform: scale(0.97);
}

.btn-cancel {
    background: rgba(255, 255, 255, 0.08);
    color: #cfcfd4;
}

.btn-cancel:hover {
    background: rgba(255, 255, 255, 0.14);
}

.btn-confirm {
    background: linear-gradient(180deg, #1d7a38 0%, #145c28 100%);
    color: #d9ffe4;
    box-shadow: 0 4px 16px rgba(35, 164, 70, 0.35);
}

.btn-confirm:hover {
    background: linear-gradient(180deg, #239347 0%, #1a6e32 100%);
}

.popup-enter-active,
.popup-leave-active {
    transition: opacity 0.22s ease;
}

.popup-enter-from,
.popup-leave-to {
    opacity: 0;
}

.popup-enter-active .popup-box,
.popup-leave-active .popup-box {
    transition: transform 0.22s ease;
}

.popup-enter-from .popup-box,
.popup-leave-to .popup-box {
    transform: translateY(18px);
}
</style>
