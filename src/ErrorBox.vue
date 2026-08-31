<script setup lang="ts">
import { computed, ref, watch, withDefaults } from "vue";

const props = withDefaults(
    defineProps<{
        message: string;
        title?: string;
        variant?: "error" | "success";
        confirmText?: string;
        cancelText?: string;
        showCancel?: boolean;
        open?: boolean;
    }>(),
    {
        variant: "error",
        confirmText: "Retry",
        cancelText: "Cancel",
        showCancel: true,
        open: true,
    }
);

const emit = defineEmits<{
    (e: "confirm"): void;
    (e: "cancel"): void;
}>();

const icon = computed(() => (props.variant === "success" ? "✓" : "!"));
const isError = computed(() => props.variant === "error");

// Own the show/hide internally so the leave transition can play before the
// overlay unmounts (it must stay in the DOM long enough to animate out).
const visible = ref(props.open);
watch(
    () => props.open,
    (o) => {
        visible.value = o;
    }
);
</script>

<template>
    <Teleport to="body">
        <Transition name="popup">
            <div v-if="visible" class="popup-overlay" @click.self="emit('cancel')">
                <div class="popup-box">
                    <div class="popup-icon" :class="{ success: !isError }" aria-hidden="true">
                        {{ icon }}
                    </div>
                    <p v-if="title" class="popup-title">{{ title }}</p>
                    <p class="popup-message">{{ message }}</p>
                    <div class="popup-actions">
                        <button v-if="showCancel" type="button" class="btn btn-cancel" @click="emit('cancel')">
                            {{ cancelText }}
                        </button>
                        <button type="button" class="btn btn-confirm" @click="emit('confirm')">
                            {{ confirmText }}
                        </button>
                    </div>
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
    gap: 14px;
}

.popup-icon {
    width: 52px;
    height: 52px;
    border-radius: 50%;
    background: radial-gradient(circle at 30% 25%, #ed4444 0%, #b81a15 70%);
    color: #fff;
    font-family: "Product Sans", "Roboto", sans-serif;
    font-size: 30px;
    font-weight: 700;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 18px rgba(184, 26, 21, 0.45);
}

.popup-icon.success {
    background: radial-gradient(circle at 30% 25%, #2aa84f 0%, #176037 70%);
    box-shadow: 0 4px 18px rgba(35, 164, 70, 0.45);
}

.popup-title {
    margin: 0;
    color: #f4f4f8;
    font-family: "Roboto", sans-serif;
    font-size: 16px;
    font-weight: 700;
    text-align: center;
}

.popup-message {
    margin: 0;
    color: #d6d6db;
    font-family: "Roboto", sans-serif;
    font-size: 14px;
    line-height: 1.5;
    text-align: center;
    word-break: break-word;
    max-height: 104px;
    overflow-y: auto;
}

.popup-actions {
    display: flex;
    gap: 12px;
    width: 100%;
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
    transition: transform 0.1s ease, background 0.2s ease;
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
