import { ref, type Ref } from "vue";

export interface MessageBoxOptions {
    message: string;
    title?: string;
    variant?: "error" | "success";
    confirmText?: string;
    showCancel?: boolean;
    onConfirm?: () => void;
}

interface MessageBoxState {
    message: Ref<string | null>;
    title: Ref<string>;
    variant: Ref<"error" | "success">;
    confirmText: Ref<string>;
    showCancel: Ref<boolean>;
    onConfirm: (() => void) | null;
}

const state: MessageBoxState = {
    message: ref<string | null>(null),
    title: ref(""),
    variant: ref<"error" | "success">("error"),
    confirmText: ref("Retry"),
    showCancel: ref(true),
    onConfirm: null,
};

export function useMessageBox() {
    function show(opts: MessageBoxOptions) {
        state.message.value = opts.message;
        state.title.value = opts.title ?? "";
        state.variant.value = opts.variant ?? "error";
        state.confirmText.value = opts.confirmText ?? "Retry";
        state.showCancel.value = opts.showCancel ?? true;
        state.onConfirm = opts.onConfirm ?? null;
    }

    function hide() {
        state.message.value = null;
        state.onConfirm = null;
    }

    function confirm() {
        const action = state.onConfirm;
        hide();
        action?.();
    }

    return {
        message: state.message,
        title: state.title,
        variant: state.variant,
        confirmText: state.confirmText,
        showCancel: state.showCancel,
        show,
        hide,
        confirm,
    };
}
