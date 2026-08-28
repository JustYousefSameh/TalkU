<script setup lang="ts">
import { Settings, Minus, X } from "lucide-vue-next";
function closeWindow() {}

function minimizeWindow() {}

function openSettings() {}
</script>

<template>
    <main>
        <div
            class="flex flex-row justify-between items-center w-full absolute mt-3 pl-3 pr-4 pointer-events-auto appbar-slide"
        >
            <div>
                <span class="app-title text-gray-300 pr-1 text-sm">TalkU</span>
                <span class="text-gray-500 text-xs">V2.4</span>
            </div>
            <div class="flex flex-row gap-5">
                <button class="text-white cursor-pointer" @click="openSettings">
                    <Settings class="h-5" />
                </button>
                <button
                    class="text-white cursor-pointer"
                    @click="minimizeWindow"
                >
                    <Minus class="h-5" />
                </button>
                <button class="text-white cursor-pointer" @click="closeWindow">
                    <X class="h-5" />
                </button>
            </div>
        </div>
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
</style>
