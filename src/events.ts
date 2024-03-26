export type EventLog = {
    id: number
    timestamp: number
    date: string
    kind: string
}

export type ApplicationLog = {
    id: number
    event_id: number
    timestamp: number
    date: string
    info_id: number
    process_id?: number
    title?: string
    x?: number
    y?: number
    width?: number
    height?: number
    ref_id?: number
}

export type ApplicationInfo = {
    id: number
    path?: string
    name?: string
}

export type BrowserLog = {
    id: number
    event_id: number
    timestamp: number
    date: string
    info_id: number
    title?: string
    referrer?: string
    tab_id?: number
    opener_tab_id?: number
    window_id?: number
}

export type BrowserInfo = {
    id: number
    url: string
    fav_icon_url?: string
}

export type FileLog = {
    id: number
    event_id: number
    timestamp: number
    date: string
    path?: string
    kind?: string
    file_type?: string
}
