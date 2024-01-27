import { writable } from "svelte/store";

type GlobalModalState = {
    open: boolean,
    title: string,
    message: string,
    isStaticBackdrop: boolean,
    onConfirm: () => void,
    confirmButtonVariant: "primary" | "secondary" | "danger"
    confirmButtonText: string,
};

const closedState: GlobalModalState = {
    open: false,
    title: "",
    message: "",
    isStaticBackdrop: false,
    onConfirm: () => { },
    confirmButtonVariant: "primary",
    confirmButtonText: "Confirm"
};

export const globalModalState = writable<GlobalModalState>(closedState);


type OpenGlobalModal = {
    title: string,
    message: string,
    isStaticBackdrop?: boolean,
    onConfirm: () => void,
    autoCloseOnConfirm?: boolean,
    confirmButtonVariant?: "primary" | "secondary" | "danger"
    confirmButtonText: string,
};

export function openGlobaModal({ title, message, onConfirm, isStaticBackdrop = false, autoCloseOnConfirm = true, confirmButtonVariant = "primary", confirmButtonText = "Confirm" }: OpenGlobalModal) {
    globalModalState.set({
        open: true,
        title: title,
        message: message,
        isStaticBackdrop: isStaticBackdrop,
        onConfirm() {
            if (autoCloseOnConfirm) {
                globalModalState.set(closedState);
            }
            onConfirm();
        },
        confirmButtonVariant,
        confirmButtonText,
    });
}