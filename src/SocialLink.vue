<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

defineProps<{
    image: string;
    alt?: string;
    link: string;
}>();

function openLink(url: string) {
    invoke("plugin:opener|open_url", { url });
}
</script>

<template>
    <button class="social-link" type="button" @click="openLink(link)">
        <img :src="image" :alt="alt ?? 'social link'" class="social-icon" />
    </button>
</template>

<style scoped>
.social-link {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    transition: transform 0.2s ease;
    outline: none;
}

.social-link:hover {
    transform: scale(1.15) rotate(-4deg);
}

.social-icon {
    display: block;
    width: 32px;
    height: 32px;
    filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.5));
}
</style>
