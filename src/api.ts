import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  BatchResult,
  ConvertOptions,
  ConvertSuccess,
  ImageInfo,
} from "./types";

const IMAGE_FILTERS = [
  {
    name: "Images",
    extensions: [
      "png",
      "jpg",
      "jpeg",
      "webp",
      "gif",
      "bmp",
      "tif",
      "tiff",
      "avif",
    ],
  },
];

export async function pickImageFiles(): Promise<string[]> {
  const selected = await open({
    multiple: true,
    directory: false,
    filters: IMAGE_FILTERS,
  });
  if (!selected) return [];
  return Array.isArray(selected) ? selected : [selected];
}

export async function pickOutputFolder(): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (!selected || Array.isArray(selected)) return null;
  return selected;
}

export function probeImage(path: string): Promise<ImageInfo> {
  return invoke("probe_image", { path });
}

export function previewImage(path: string, maxEdge = 480): Promise<string> {
  return invoke("preview_image", { path, maxEdge });
}

export function convertImage(
  sourcePath: string,
  outputDir: string,
  options: ConvertOptions,
): Promise<ConvertSuccess> {
  return invoke("convert_image", { sourcePath, outputDir, options });
}

export function convertBatch(
  sourcePaths: string[],
  outputDir: string,
  options: ConvertOptions,
): Promise<BatchResult> {
  return invoke("convert_batch", { sourcePaths, outputDir, options });
}
