import { ref, onMounted } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import { useMessageBox } from "../stores/messageBox";

/** GitHub page of the newest published TalkU release. */
export const LATEST_RELEASE_URL =
    "https://github.com/JustYousefSameh/TalkU/releases/latest";

/** Bytes downloaded so far, sent by Rust after every chunk. */
export interface DownloadProgress {
    received: number;
    total: number;
}

/** Which screen of the update dialog is showing, or `null` when it's closed. */
export type UpdateStep = "prompt" | "downloading" | "failed";

/**
 * Looks for a newer TalkU release when the app starts and offers to install
 * it. If the user picks "Later", `newVersion` stays set so the app bar can
 * keep showing a small "new version" button.
 */
export function useUpdateCheck() {
    const newVersion = ref<string | null>(null);
    const step = ref<UpdateStep | null>(null);
    const progress = ref<DownloadProgress>({ received: 0, total: 0 });
    // Average download speed in bytes per second.
    const speed = ref(0);
    const error = ref("");

    function promptUpdate() {
        // The download can't be restarted halfway; ignore the app bar button
        // (still reachable with the keyboard) until it's done.
        if (step.value !== "downloading") step.value = "prompt";
    }

    function closeDialog() {
        step.value = null;
    }

    async function install() {
        // Start timing at the first byte, so looking up the release and
        // connecting to GitHub don't drag the speed down.
        let startedAt: number | undefined;
        progress.value = { received: 0, total: 0 };
        speed.value = 0;
        step.value = "downloading";

        const onProgress = new Channel<DownloadProgress>();
        onProgress.onmessage = (p) => {
            startedAt ??= performance.now();
            const seconds = (performance.now() - startedAt) / 1000;
            progress.value = p;
            speed.value = seconds > 0 ? p.received / seconds : 0;
        };

        try {
            // On success TalkU closes and the installer takes over.
            await invoke("install_update", { onProgress });
        } catch (err) {
            error.value = String(err);
            step.value = "failed";
        }
    }

    onMounted(async () => {
        try {
            newVersion.value = await invoke<string | null>("check_for_update");
        } catch (err) {
            console.error("Failed to check for updates:", err);
            return;
        }
        // If another popup is already open (like a connection error), don't
        // pile on top of it. The app bar button still shows the update.
        if (newVersion.value && useMessageBox().message.value === null) {
            promptUpdate();
        }
    });

    return {
        newVersion,
        step,
        progress,
        speed,
        error,
        promptUpdate,
        closeDialog,
        install,
    };
}
