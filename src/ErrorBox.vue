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
                <button type="button" class="btn btn-cancel" @click="emit('cancel')">
                    Cancel
                </button>
                <button type="button" class="btn btn-retry" @click="emit('retry')">
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
    background: rgba(0, 0, 0, 0.6);
    z-index: 100;
}

.error-box {
    width: 300px;
    max-width: 88vw;
    padding: 20px;
    border-radius: 16px;
    background: #1d1d1f;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
}

.error-icon {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: #b81a15;
    color: #fff;
    font-size: 28px;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
}

.error-message {
    margin: 0;
    color: #e8e8ea;
    font-size: 14px;
    line-height: 1.4;
    text-align: center;
    word-break: break-word;
    max-height: 120px;
    overflow-y: auto;
}

.error-actions {
    display: flex;
    gap: 10px;
    width: 100%;
}

.btn {
    flex: 1;
    padding: 9px 0;
    border: none;
    border-radius: 10px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
}

.btn-cancel {
    background: #2c2c2f;
    color: #cfcfd4;
}

.btn-cancel:hover {
    background: #3a3a3e;
}

.btn-retry {
    background: #0c2810;
    color: #23a446;
}

.btn-retry:hover {
    background: #143c1c;
}
</style>
