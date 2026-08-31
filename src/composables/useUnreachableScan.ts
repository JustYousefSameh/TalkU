import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useMessageBox } from "../stores/messageBox";

/** How long (seconds) an unreachable scan runs on the daemon side. */
const SCAN_DURATION_SECS = 30;

/**
 * App-level state and logic for the unreachable-IP scan. Lives in a composable
 * (rather than inside the Monitor menu) so the scan keeps running and can notify
 * the user even after the Monitor menu is closed — the daemon scans in the
 * background regardless; we only time the UI locally.
 */
export function useUnreachableScan() {
    const scanning = ref<string | null>(null);
    const messageBox = useMessageBox();
    let timer: ReturnType<typeof setTimeout> | null = null;

    function clearTimer() {
        if (timer) {
            clearTimeout(timer);
            timer = null;
        }
    }

    async function start(name: string) {
        if (scanning.value) return; // only one scan at a time
        clearTimer();
        scanning.value = name;
        try {
            await invoke("collect_unreachable", {
                processName: name,
                seconds: SCAN_DURATION_SECS,
            });
            messageBox.show({
                message: `Scan initiated for ${name}. Please reproduce the issue so we can capture the logs.`,
                variant: "success",
                confirmText: "OK",
                showCancel: false,
            });
            // Notify when the monitoring window ends. Client-side timer so it
            // works even if the menu is closed meanwhile.
            timer = setTimeout(() => {
                if (scanning.value === name) {
                    messageBox.show({
                        message: `Monitoring for ${name} has ended (${SCAN_DURATION_SECS}s).`,
                        variant: "success",
                        confirmText: "OK",
                        showCancel: false,
                    });
                    scanning.value = null;
                }
            }, SCAN_DURATION_SECS * 1000);
        } catch (e) {
            messageBox.show({
                message: String(e),
                variant: "error",
                confirmText: "OK",
                showCancel: false,
            });
            clearTimer();
            scanning.value = null;
        }
    }

    function dispose() {
        clearTimer();
    }

    return {
        scanning,
        start,
        dispose,
    };
}
