import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const enabled = ref(true);
let loaded = false;

export function useAudioCues() {
    async function load() {
        if (loaded) return;
        try {
            enabled.value = await invoke<boolean>("get_audio_cues");
            loaded = true;
        } catch (err) {
            console.error("Failed to load audio cues setting:", err);
        }
    }

    async function set(next: boolean) {
        try {
            await invoke("set_audio_cues", { enabled: next });
            enabled.value = next;
        } catch (err) {
            console.error("Failed to update audio cues setting:", err);
        }
    }

    return { enabled, load, set };
}