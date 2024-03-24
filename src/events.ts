export type ApplicationLog = {
    id: number
    event_id: number
    timestamp: number
    date: string
    process_id?: number
    name?: string
    title?: string
    x?: number
    y?: number
    width?: number
    height?: number
    ref_id?: number
}

export type BrowserLog = {
    id: number
    event_id: number
    timestamp: number
    date: string
    tab_id?: number
    url?: string
    title?: string
    fav_icon_url?: string
    referrer?: string
    opener_tab_id?: number
    window_id?: number
}
