export type EventLog = {
    id: number
    timestamp: number
    date: string
    kind: string
}

export type ApplicationLog = {
    id: number
    timestamp: number
    date: string
    path: string
    name?: string
    process_id?: number
    title?: string
    x?: number
    y?: number
    width?: number
    height?: number
    ref_id?: number
}

export type BrowserLog = {
    id: number
    timestamp: number
    date: string
    url: string
    title?: string
    fav_icon_url?: string
    referrer?: string
    tab_id?: number
    opener_tab_id?: number
    window_id?: number
}

export type FileLog = {
    id: number
    timestamp: number
    date: string
    path: string
    kind?: string
}

export type ScreenshotLog = {
    id: number
    timestamp: number
    date: string
    monitor_id: number
}
