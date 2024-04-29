import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

// function timestamp_mm(timestamp: number): string {
//   let date = new Date(timestamp * 1000);
//   return ("0" + date.toLocaleTimeString("ja-JP", { minute: "numeric" })).slice(-2);
// }

export function timestamp_yyyymmss(timestamp: number): string {
  let date = new Date(timestamp * 1000);
  return date.toLocaleDateString();
}

export function timestamp_mmss(timestamp: number): string {
  let date = new Date(timestamp * 1000);
  return ("0" + date.toLocaleTimeString("ja-JP", { minute: "2-digit", second: "2-digit" })).slice(-5);
}

export function timestamp_hhmm(timestamp: number): string {
  let date = new Date(timestamp * 1000);
  return ("0" + date.toLocaleTimeString("ja-JP", { hour: "2-digit", minute: "2-digit"})).slice(-5);
}

export function filename(path: string): string {
  return path.split(/[/\\]/).pop() || "";
}
