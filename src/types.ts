export type OutputFormat =
  | "webp"
  | "avif"
  | "jpeg"
  | "png"
  | "gif"
  | "bmp"
  | "tiff";

export interface ConvertOptions {
  format: OutputFormat;
  quality: number;
  lossless: boolean;
  background?: [number, number, number];
  suffix?: string;
}

export interface ImageInfo {
  path: string;
  fileName: string;
  width: number;
  height: number;
  format: string;
  sizeBytes: number;
  hasAlpha: boolean;
}

export interface ConvertSuccess {
  sourcePath: string;
  outputPath: string;
  inputBytes: number;
  outputBytes: number;
  width: number;
  height: number;
  format: string;
}

export interface ConvertFailure {
  sourcePath: string;
  error: string;
}

export interface BatchResult {
  successes: ConvertSuccess[];
  failures: ConvertFailure[];
}

export interface QueueItem {
  id: string;
  path: string;
  info?: ImageInfo;
  previewUrl?: string;
  status: "pending" | "ready" | "converting" | "done" | "error";
  error?: string;
  result?: ConvertSuccess;
}

export const FORMATS: { id: OutputFormat; label: string; lossless: boolean }[] = [
  { id: "webp", label: "WebP", lossless: true },
  { id: "avif", label: "AVIF", lossless: true },
  { id: "jpeg", label: "JPEG", lossless: false },
  { id: "png", label: "PNG", lossless: true },
  { id: "gif", label: "GIF", lossless: false },
  { id: "bmp", label: "BMP", lossless: true },
  { id: "tiff", label: "TIFF", lossless: true },
];

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}
