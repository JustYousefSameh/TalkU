<script setup lang="ts">
defineProps<{
    message: string;
}>();

const emit = defineEmits<{
    (e: "retry"): void;
    (e: "cancel"): void;
}>();
</script>

<template>
    <div class="error-overlay">
        <div class="error-box">
            <div class="error-icon" aria-hidden="true">!</div>
            <p class="error-message">{{ message }}</p>
            <div class="error-actions">
                <button
                    type="button"
                    class="btn btn-cancel"
                    @click="emit('cancel')"
                >
                    Cancel
                </button>
                <button
                    type="button"
                    class="btn btn-retry"
                    @click="emit('retry')"
                >
                    Retry
                </button>
            </div>
        </div>
    </div>
</template>

<style scoped>
.error-overlay {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.55);
    border-radius: 12px;
    overflow: hidden;
    z-index: 100;
}

.error-box {
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

.error-icon {
    width: 52px;
    height: 52px;
    border-radius: 50%;
    background: radial-gradient(
        circle at 30% 25%,
        #ed4444 0%,
        #b81a15 70%
    );
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

.error-message {
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

.error-actions {
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

.btn-retry {
    background: linear-gradient(180deg, #1d7a38 0%, #145c28 100%);
    color: #d9ffe4;
    box-shadow: 0 4px 16px rgba(35, 164, 70, 0.35);
}

.btn-retry:hover {
    background: linear-gradient(180deg, #239347 0%, #1a6e32 100%);
}
</style>
