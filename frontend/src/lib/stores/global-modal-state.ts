import { writable } from "svelte/store";

type GlobalModalState = {
    open: boolean,
    title: string,
    message: string,
    isStaticBackdrop: boolean,
    onConfirm: () => void,
};

const closedState = {
    open: false,
    title: "",
    message: "",
    isStaticBackdrop: false,
    onConfirm: () => { },
};

export const globalModalState = writable<GlobalModalState>(closedState);


type OpenGlobalModal = {
    title: string,
    message: string,
    isStaticBackdrop?: boolean,
    onConfirm: () => void,
    autoCloseOnConfirm?: boolean
};

export function openGlobaModal({ title, message, onConfirm, isStaticBackdrop = false, autoCloseOnConfirm = true }: OpenGlobalModal) {
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
    });
}