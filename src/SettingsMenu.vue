<script setup lang="ts">
import { ref } from "vue";
import { Settings, Gamepad2, Plus, Check, X, Trash2 } from "lucide-vue-next";

defineProps<{
    open: boolean;
}>();

const games = ref<string[]>([]);
const addingGame = ref(false);
const newGame = ref("");
const inputRef = ref<HTMLInputElement | null>(null);

function showAdd() {
    addingGame.value = true;
    newGame.value = "";
    requestAnimationFrame(() => inputRef.value?.focus());
}

function confirmAdd() {
    const game = newGame.value.trim();
    if (game && !games.value.includes(game)) {
        games.value.push(game);
    }
    addingGame.value = false;
    newGame.value = "";
}

function cancelAdd() {
    addingGame.value = false;
    newGame.value = "";
}

function removeGame(game: string) {
    games.value = games.value.filter((g) => g !== game);
}
</script>

<template>
    <Transition name="menu-fade">
        <div
            v-if="open"
            class="settings-menu"
            data-tauri-drag-region="false"
            @click.stop
        >
            <div class="menu-head">
                <Settings class="h-4 menu-head-icon" />
                <span>Settings</span>
            </div>

            <div class="menu-body">
                <div class="menu-group-title">General</div>
                <button class="menu-item" type="button">
                    <div class="menu-item-label">
                        <span>Launch on startup</span>
                        <span class="menu-item-desc"
                            >Open TalkU when you log in</span
                        >
                    </div>
                    <span class="toggle-pill"
                        ><span class="toggle-pill-dot"></span
                    ></span>
                </button>
                <button class="menu-item" type="button">
                    <div class="menu-item-label">
                        <span>Auto connect on game launch</span>
                        <span class="menu-item-desc"
                            >Connect the tunnel when a game starts</span
                        >
                    </div>
                    <span class="toggle-pill"
                        ><span class="toggle-pill-dot"></span
                    ></span>
                </button>

                <div class="menu-group-title">Monitored games</div>
                <div class="games-box">
                    <div
                        v-for="game in games"
                        :key="game"
                        class="game-row"
                    >
                        <span class="game-row-icon"
                            ><Gamepad2 class="h-3.5" /></span
                        >
                        <span class="game-row-name">{{ game }}</span>
                        <span class="menu-item-value">On</span>
                        <button
                            class="game-row-remove"
                            type="button"
                            title="Remove game"
                            @click="removeGame(game)"
                        >
                            <Trash2 class="h-3.5" />
                        </button>
                    </div>

                    <div v-if="addingGame" class="game-add-input-row">
                        <input
                            ref="inputRef"
                            v-model="newGame"
                            class="game-add-input"
                            type="text"
                            placeholder="game.exe"
                            @keydown.enter.prevent="confirmAdd"
                            @keydown.esc.prevent="cancelAdd"
                        />
                        <button
                            class="game-add-confirm"
                            type="button"
                            title="Add"
                            @click="confirmAdd"
                        >
                            <Check class="h-4" />
                        </button>
                        <button
                            class="game-add-cancel"
                            type="button"
                            title="Cancel"
                            @click="cancelAdd"
                        >
                            <X class="h-4" />
                        </button>
                    </div>

                    <div
                        v-if="games.length === 0 && !addingGame"
                        class="games-empty"
                    >
                        <Gamepad2 class="h-5 games-empty-icon" />
                        <span>No games monitored yet</span>
                    </div>

                    <div class="games-footer">
                        <button
                            class="games-add-btn"
                            type="button"
                            title="Add a game executable"
                            @click="showAdd"
                        >
                            <Plus class="h-4" />
                            <span>Add game</span>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </Transition>
</template>

<style scoped>
.settings-menu {
    font-family: "Roboto", sans-serif;
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -47%);
    display: flex;
    flex-direction: column;
    width: 280px;
    max-height: calc(100vh - 48px);
    z-index: 1000;
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
    padding: 9px 14px;
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

.menu-group-title {
    padding: 8px 12px 2px;
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #6b7280;
}

.menu-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    width: 100%;
    padding: 7px 10px;
    border: none;
    background: transparent;
    border-radius: 8px;
    color: #e5e7eb;
    text-align: left;
    cursor: pointer;
    transition: background-color 0.14s ease;
}

.menu-item:hover {
    background: rgba(255, 255, 255, 0.06);
}

.menu-item-label {
    display: flex;
    flex-direction: column;
    gap: 0px;
    flex: 1;
    font-size: 12px;
    font-weight: 500;
    line-height: 1.25;
}

.menu-item-desc {
    font-size: 10.5px;
    color: #8b919a;
}

.menu-item-value {
    font-size: 11px;
    color: #23a446;
    white-space: nowrap;
}

.toggle-pill {
    position: relative;
    width: 30px;
    height: 17px;
    border-radius: 9999px;
    background: #23a446;
    flex-shrink: 0;
}

.toggle-pill-dot {
    position: absolute;
    top: 2px;
    right: 2px;
    width: 13px;
    height: 13px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
}

/* Scrollable box that lists the game executables being watched. */
.games-box {
    margin: 2px 6px 6px;
    max-height: 190px;
    overflow-y: auto;
    overflow-x: hidden;
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.02);
    scrollbar-width: thin;
    scrollbar-color: rgba(35, 164, 70, 0.5) transparent;
}

.games-box::-webkit-scrollbar {
    width: 6px;
}

.games-box::-webkit-scrollbar-thumb {
    background: rgba(35, 164, 70, 0.5);
    border-radius: 9999px;
}

.games-box::-webkit-scrollbar-track {
    background: transparent;
}

.game-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 10px;
    font-size: 12px;
    color: #e5e7eb;
}

.game-row + .game-row {
    border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.game-row-icon {
    color: #6b7280;
    display: inline-flex;
}

.game-row-name {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.game-row-remove {
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

.game-row-remove:hover {
    color: #f87171;
    background: rgba(248, 113, 113, 0.15);
}

.games-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 18px 12px;
    color: #8b919a;
    font-size: 11.5px;
}

.games-empty-icon {
    color: #6b7280;
}

.games-add-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 6px 12px;
    border: 1px solid rgba(35, 164, 70, 0.4);
    border-radius: 7px;
    background: rgba(35, 164, 70, 0.1);
    color: #23a446;
    font-size: 11px;
    cursor: pointer;
    transition:
        background-color 0.14s ease,
        border-color 0.14s ease;
}

.games-add-btn:hover {
    background: rgba(35, 164, 70, 0.18);
    border-color: rgba(35, 164, 70, 0.6);
}

.games-footer {
    display: flex;
    justify-content: center;
    padding: 8px 10px 10px;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.game-add-input-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 10px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.game-add-input {
    flex: 1;
    min-width: 0;
    padding: 6px 9px;
    border: 1px solid rgba(35, 164, 70, 0.4);
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.04);
    color: #e5e7eb;
    font-size: 11.5px;
    font-family: "Roboto", sans-serif;
}

.game-add-input:focus {
    outline: none;
    border-color: #23a446;
}

.game-add-confirm,
.game-add-cancel {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.04);
    color: #e5e7eb;
    cursor: pointer;
    transition:
        background-color 0.14s ease,
        border-color 0.14s ease;
    flex-shrink: 0;
}

.game-add-confirm {
    color: #23a446;
}

.game-add-confirm:hover {
    background: rgba(35, 164, 70, 0.18);
    border-color: rgba(35, 164, 70, 0.6);
}

.game-add-cancel:hover {
    background: rgba(248, 113, 113, 0.15);
    border-color: rgba(248, 113, 113, 0.5);
    color: #f87171;
}

.menu-fade-enter-active,
.menu-fade-leave-active {
    transition: opacity 0.14s ease;
}

.menu-fade-enter-from,
.menu-fade-leave-to {
    opacity: 0;
}
</style>
