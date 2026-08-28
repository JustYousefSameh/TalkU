<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const connectedUsers = ref<number>(0);

onMounted(async () => {
    try {
        connectedUsers.value = await invoke<number>(
            "get_connected_users_count",
        );
    } catch (err) {
        console.error(err);
    }
});
</script>

<template>
    <main
        class="flex flex-row fixed bottom-0 right-0 gap-1 justify-center items-center p-4 users-slide"
    >
        <div class="dot"></div>
        <span class="text-gray-400 text-xs overflow-hidden leading-none">
            <Transition name="count-flow" mode="out-in">
                <span :key="connectedUsers" class="inline-block">{{
                    connectedUsers
                }}</span>
            </Transition>
        </span>
    </main>
</template>

<style scoped>
.users-slide {
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

.dot {
    width: 8px;
    height: 8px;
    background-color: rgba(74, 222, 128, 0.8);
    border-radius: 50%;
    display: inline-block;
    animation: pulse 3s ease-in-out infinite;
    box-shadow: 0 0 6px 1px rgba(74, 222, 128, 0.35);
}

.count-flow-enter-active {
    transition: all 0.35s ease;
}

.count-flow-leave-active {
    transition: all 0.25s ease;
}

.count-flow-leave-to {
    opacity: 0;
    transform: translateY(-14px);
}

.count-flow-enter-from {
    opacity: 0;
    transform: translateY(14px);
}

/*Smooth glowing pulse animation*/
@keyframes pulse {
    0%,
    100% {
        opacity: 0.5;
        box-shadow: 0 0 3px 0 rgba(74, 222, 128, 0.25);
    }
    50% {
        opacity: 0.9;
        box-shadow: 0 0 9px 2px rgba(74, 222, 128, 0.5);
    }
}
</style>
