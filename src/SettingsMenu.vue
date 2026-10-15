<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Settings, Gamepad2, Plus, Check, X, Trash2 } from "lucide-vue-next";
import { useAudioCues } from "./stores/audioCues";

defineProps<{
    open: boolean;
}>();

defineEmits<{
    "open-monitor": [];
}>();

interface SupportedGame {
    id: string;
    name: string;
    process_names: string[];
    steam_app_ids: number[];
    epic_names: string[];
    xbox_names: string[];
}

interface MonitoredGame {
    name: string;
    process_name: string;
}

interface GameInstallation {
    found: boolean;
    source: string;
    install_dir: string | null;
    executable: string | null;
    process_name: string | null;
}

const games = ref<MonitoredGame[]>([]);
const supportedGames = ref<SupportedGame[]>([]);
const addingGame = ref(false);
const selectedGameId = ref("");
const customGameName = ref("");
const customDisplayName = ref("");
const detection = ref<GameInstallation | null>(null);
const detecting = ref(false);
let detectionRequestId = 0;
const fileInput = ref<HTMLInputElement | null>(null);
const launchOnStartup = ref(false);
const autoConnect = ref(false);

const { enabled: audioCues, load: loadAudioCues, set: setAudioCues } = useAudioCues();

onMounted(async () => {
    try {
        launchOnStartup.value = await invoke<boolean>("get_launch_on_startup");
    } catch (err) {
        console.error("Failed to load autostart setting:", err);
    }
    try {
        autoConnect.value = await invoke<boolean>("get_auto_connect");
        games.value = await invoke<MonitoredGame[]>("get_monitored_games");
        supportedGames.value = await invoke<SupportedGame[]>("get_supported_games");
    } catch (err) {
        console.error("Failed to load game settings:", err);
    }
    loadAudioCues();
});

async function toggleLaunchOnStartup() {
    const next = !launchOnStartup.value;
    try {
        await invoke("set_launch_on_startup", { enabled: next });
        launchOnStartup.value = next;
    } catch (err) {
        console.error("Failed to update autostart setting:", err);
    }
}

async function toggleAutoConnect() {
    const next = !autoConnect.value;
    try {
        await invoke("set_auto_connect", { enabled: next });
        autoConnect.value = next;
    } catch (err) {
        console.error("Failed to update auto-connect setting:", err);
    }
}

async function toggleAudioCues() {
    await setAudioCues(!audioCues.value);
}

function showAdd() {
    detectionRequestId++;
    addingGame.value = true;
    detection.value = null;
    detecting.value = false;
    customGameName.value = "";
    customDisplayName.value = "";
    selectedGameId.value = supportedGames.value[0]?.id ?? "";
    if (selectedGameId.value) detectSelectedGame();
}

function onSelectedGameChange() {
    if (selectedGameId.value === "__other__") {
        detectionRequestId++;
        detecting.value = false;
        detection.value = null;
        customGameName.value = "";
        customDisplayName.value = "";
        return;
    }

    customGameName.value = "";
    customDisplayName.value = "";
    detectSelectedGame();
}

async function detectSelectedGame() {
    const requestId = ++detectionRequestId;
    if (!selectedGameId.value || selectedGameId.value === "__other__") {
        detecting.value = false;
        return;
    }
    detecting.value = true;
    detection.value = null;

    try {
        const result = await invoke<GameInstallation>("detect_game_installation", {
            gameId: selectedGameId.value,
        });
        if (requestId !== detectionRequestId) return;
        detection.value = result;
    } catch (err) {
        if (requestId !== detectionRequestId) return;
        console.error("Failed to detect game:", err);
        detection.value = {
            found: false,
            source: "Detection failed",
            install_dir: null,
            executable: null,
            process_name: null,
        };
    } finally {
        if (requestId === detectionRequestId) {
            detecting.value = false;
        }
    }
}

function browseForGame() {
    fileInput.value?.click();
}

function handleFileSelected(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    selectedGameId.value = "__other__";
    const processName = file.name;
    customGameName.value = processName;
    if (!customDisplayName.value.trim()) {
        customDisplayName.value = processName.replace(/\.exe$/i, "");
    }
    detection.value = {
        found: true,
        source: "Manual selection",
        install_dir: null,
        executable: processName,
        process_name: processName,
    };
    input.value = "";
}

function handleManualProcessNameChange() {
    const processName = customGameName.value.trim();

    if (!processName) {
        detection.value = null;
        return;
    }

    detection.value = {
        found: true,
        source: "Manual entry",
        install_dir: null,
        executable: processName,
        process_name: processName,
    };
    if (!customDisplayName.value.trim()) {
        customDisplayName.value = processName.replace(/\.exe$/i, "");
    }
}

async function confirmAdd() {
    const processName = detection.value?.process_name?.trim();
    if (
        !processName ||
        games.value.some(
            (g) => g.process_name.toLowerCase() === processName.toLowerCase(),
        )
    ) {
        return;
    }

    const selected = supportedGames.value.find((g) => g.id === selectedGameId.value);
    const defaultCustomName = processName.replace(/\.exe$/i, "");
    const displayName =
        selectedGameId.value === "__other__"
            ? customDisplayName.value.trim() || defaultCustomName
            : selected?.name ?? processName;

    try {
        await invoke("add_monitored_game", { name: processName, displayName });
        games.value.push({ name: displayName, process_name: processName });
        addingGame.value = false;
        selectedGameId.value = "";
        customGameName.value = "";
        customDisplayName.value = "";
        detection.value = null;
    } catch (err) {
        console.error("Failed to add game:", err);
    }
}

function cancelAdd() {
    detectionRequestId++;
    addingGame.value = false;
    selectedGameId.value = "";
    customGameName.value = "";
    customDisplayName.value = "";
    detection.value = null;
    detecting.value = false;
}

async function removeGame(game: MonitoredGame) {
    try {
        await invoke("remove_monitored_game", { name: game.process_name });
        games.value = games.value.filter((g) => g.process_name !== game.process_name);
    } catch (err) {
        console.error("Failed to remove game:", err);
    }
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
                <button
                    class="menu-item"
                    type="button"
                    @click="toggleLaunchOnStartup"
                >
                    <div class="menu-item-label">
                        <span>Launch on startup</span>
                        <span class="menu-item-desc"
                            >Open TalkU when you log in</span
                        >
                    </div>
                    <span class="toggle-pill" :class="{ on: launchOnStartup }"
                        ><span class="toggle-pill-dot"></span
                    ></span>
                </button>
                <button
                    class="menu-item"
                    type="button"
                    @click="toggleAudioCues"
                >
                    <div class="menu-item-label">
                        <span>Audio cues</span>
                        <span class="menu-item-desc"
                            >Play a sound when the tunnel connects or
                            disconnects</span
                        >
                    </div>
                    <span class="toggle-pill" :class="{ on: audioCues }"
                        ><span class="toggle-pill-dot"></span
                    ></span>
                </button>

                <div class="menu-group-title">Games</div>
                <button
                    class="menu-item"
                    type="button"
                    @click="toggleAutoConnect"
                >
                    <div class="menu-item-label">
                        <span>Auto connect on game launch</span>
                        <span class="menu-item-desc"
                            >Connect the tunnel when a game starts</span
                        >
                    </div>
                    <span class="toggle-pill" :class="{ on: autoConnect }"
                        ><span class="toggle-pill-dot"></span
                    ></span>
                </button>
                <div v-if="autoConnect" class="games-box">
                    <div
                        v-for="game in games"
                        :key="game.process_name"
                        class="game-row"
                    >
                        <span class="game-row-icon"
                            ><Gamepad2 class="h-3.5" /></span
                        >
                        <span class="game-row-name" :title="game.process_name">{{ game.name }}</span>
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

                    <div v-if="addingGame" class="game-add-panel">
                        <input
                            ref="fileInput"
                            type="file"
                            accept=".exe"
                            class="game-file-input"
                            @change="handleFileSelected"
                        />
                        <select
                            v-model="selectedGameId"
                            class="game-add-select"
                            @change="onSelectedGameChange"
                        >
                            <option
                                v-for="game in supportedGames"
                                :key="game.id"
                                :value="game.id"
                            >
                                {{ game.name }}
                            </option>
                            <option value="__other__">Other — Enter process name or browse</option>
                        </select>

                        <div v-if="selectedGameId === '__other__'" class="game-custom-fields">
                            <label class="game-custom-label" for="custom-game-name">
                                Display name
                            </label>
                            <input
                                id="custom-game-name"
                                v-model="customDisplayName"
                                class="game-custom-input"
                                type="text"
                                placeholder="Enter a friendly game name"
                                maxlength="80"
                            />
                            <label class="game-custom-label" for="custom-game-process">
                                Process name (use if Browse fails)
                            </label>
                            <input
                                id="custom-game-process"
                                v-model="customGameName"
                                class="game-custom-input"
                                type="text"
                                placeholder="Enter process name (e.g. game.exe)"
                                spellcheck="false"
                                @input="handleManualProcessNameChange"
                            />
                        </div>

                        <div class="game-detection">
                            <div v-if="detecting" class="game-detection-status">
                                Checking Steam, Epic Games and Xbox / Game Pass…
                            </div>
                            <template v-else-if="detection?.found">
                                <div class="game-detection-status success">
                                    <Check class="h-3.5" />
                                    {{ detection.source }} detected
                                </div>
                                <div class="game-detection-path" :title="detection.executable ?? undefined">
                                    {{ detection.executable }}
                                </div>
                            </template>
                            <div v-else class="game-detection-status">
                                {{
                                    selectedGameId === "__other__"
                                        ? "Enter the game process name or use Browse for EXE"
                                        : detection?.source ?? "No installation detected"
                                }}
                            </div>
                        </div>

                        <div class="game-add-actions">
                            <button
                                class="game-browse-btn"
                                type="button"
                                @click="browseForGame"
                            >
                                Browse for EXE
                            </button>
                            <button
                                class="game-add-confirm"
                                type="button"
                                title="Add"
                                :disabled="!detection?.process_name"
                                @click="confirmAdd"
                            >
                                <Check class="h-4" />
                                Add
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
                    </div>

                    <div
                        v-if="games.length === 0 && !addingGame"
                        class="games-empty"
                    >
                        <Gamepad2 class="h-5 games-empty-icon" />
                        <span>No games added yet</span>
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

                <div class="menu-group-title">Troubleshooting</div>
                <button
                    class="menu-item"
                    type="button"
                    @click="$emit('open-monitor')"
                >
                    <div class="menu-item-label">
                        <span>Troubleshoot connection</span>
                        <span class="menu-item-desc"
                            >Pick a process and send its logs to the server</span
                        >
                    </div>
                    <span class="menu-item-value">Open</span>
                </button>
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
    background: #3f444b;
    flex-shrink: 0;
    transition: background-color 0.18s ease;
}

.toggle-pill.on {
    background: #23a446;
}

.toggle-pill-dot {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 13px;
    height: 13px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
    transition:
        left 0.18s ease,
        right 0.18s ease;
}

.toggle-pill.on .toggle-pill-dot {
    left: auto;
    right: 2px;
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

.game-file-input {
    display: none;
}

.game-add-panel {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.025);
}

.game-add-select {
    width: 100%;
    padding: 8px 10px;
    border: 1px solid rgba(35, 164, 70, 0.55);
    border-radius: 7px;
    background: rgba(35, 164, 70, 0.16);
    color: #eaffef;
    font-weight: 500;
    color-scheme: dark;
    cursor: pointer;
}

.game-add-select:focus {
    outline: none;
    border-color: #23a446;
    box-shadow: 0 0 0 2px rgba(35, 164, 70, 0.15);
}

.game-add-select option {
    background: #151918;
    color: #eaffef;
}

.game-custom-fields {
    display: flex;
    flex-direction: column;
    gap: 5px;
}

.game-custom-label {
    color: #cbd5e1;
    font-size: 10.5px;
    font-weight: 600;
}

.game-custom-input {
    width: 100%;
    min-width: 0;
    box-sizing: border-box;
    padding: 7px 9px;
    border: 1px solid rgba(96, 165, 250, 0.55);
    border-radius: 7px;
    background: rgba(37, 99, 235, 0.10);
    color: #f3f4f6;
    font-size: 11.5px;
    font-family: "Roboto", sans-serif;
}

.game-custom-input::placeholder {
    color: #9ca3af;
}

.game-custom-input:focus {
    outline: none;
    border-color: #60a5fa;
    box-shadow: 0 0 0 2px rgba(37, 99, 235, 0.16);
}

.game-custom-exe {
    overflow: hidden;
    color: #9ca3af;
    font-size: 10px;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.game-detection {
    min-width: 0;
}

.game-detection-status {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    opacity: 0.95;
    color: #d9e6dc;
}

.game-detection-status.success {
    opacity: 1;
    color: #7ee787;
}

.game-detection-path {
    margin-top: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 10px;
    opacity: 0.85;
    color: #b8c7bc;
}

.game-add-actions {
    display: flex;
    align-items: center;
    gap: 6px;
}

.game-browse-btn {
    flex: 1;
    min-height: 30px;
    padding: 6px 9px;
    border: 1px solid rgba(96, 165, 250, 0.7);
    border-radius: 7px;
    background: #1d4ed8;
    color: #ffffff;
    font-size: 10.5px;
    font-weight: 600;
    cursor: pointer;
    transition:
        background-color 0.14s ease,
        border-color 0.14s ease;
}

.game-browse-btn:hover {
    background: #2563eb;
    border-color: #93c5fd;
}

.game-add-confirm {
    display: inline-flex;
    align-items: center;
    gap: 4px;
}

.game-add-confirm:disabled {
    opacity: 0.35;
    cursor: not-allowed;
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
    width: 30px;
    height: 30px;
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
    border-color: #15803d;
    background: #15803d;
    color: #ffffff;
}

.game-add-confirm:hover:not(:disabled) {
    border-color: #22c55e;
    background: #16a34a;
}

.game-add-confirm:disabled {
    border-color: rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.04);
    color: #9ca3af;
}

.game-add-cancel:hover {
    background: rgba(248, 113, 113, 0.15);
    border-color: rgba(248, 113, 113, 0.5);
    color: #f87171;
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
