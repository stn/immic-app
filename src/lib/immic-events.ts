import { useSyncExternalStore } from "react";
import { emit, listen } from "@tauri-apps/api/event";
import { ApplicationLog, BrowserLog, FileLog } from "./events";
import { HitsPerDay } from "./api";

const EVENT_LABEL = "immic-event";

let listeners: (() => void)[] = [];

type TEvent = {
    event: string;
    payload: ImmicEvent;
};

export type ImmicEvent = {
    Application?: [ApplicationLog, HitsPerDay[]];
    Browser?: [BrowserLog, HitsPerDay[]];
    File?: [FileLog, HitsPerDay[]];
    OpenHourly?: number;
};

const eventStore = {
    event: null as ImmicEvent | null,

    unlisten: () => {},

    emit: (event: string, payload?: ImmicEvent) => {
        emit(event, payload);
    },

    subscribe(listener: () => void) {
        listeners = [...listeners, listener];
        return () => {
            listeners = listeners.filter((l) => l !== listener);
        };
    },

    getSnapshot() {
        return eventStore.event;
    }
};

function emitChange() {
  for (let listener of listeners) {
    listener();
  }
}

listen(EVENT_LABEL, (event: TEvent) => {
    eventStore.event = event.payload;
    emitChange();
}).then((unlisten) => {
    eventStore.unlisten = unlisten;
});

export function useTauriEvent(): ImmicEvent | null {
    const event = useSyncExternalStore(eventStore.subscribe, eventStore.getSnapshot);
    return event;
}
