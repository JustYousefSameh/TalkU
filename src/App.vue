<script setup lang="ts">
import { ref } from "vue";
import AppBar from "./AppBar.vue";
import CustomSliderButton from "./CustomSliderButton.vue";
import SocialLink from "./SocialLink.vue";
import ActiveUsers from "./ActiveUsers.vue";
import GradientBackgroundWithImage from "./GradientBackgroundWithImage.vue";
import ConnectionTimer from "./ConnectionTimer.vue";
import type { Status } from "./types";

const status = ref<Status>("disconnected");
</script>

<template>
    <main>
        <div class="app-main"  data-tauri-drag-region="deep">
            <GradientBackgroundWithImage :status="status" />
            <AppBar />
            <CustomSliderButton v-model:status="status" />
            <ConnectionTimer :status="status" />
            <ActiveUsers />
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
