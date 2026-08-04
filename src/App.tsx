import { useCallback, useEffect, useMemo, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import {
  convertBatch,
  pickImageFiles,
  pickOutputFolder,
  previewImage,
  probeImage,
} from "./api";
import { FORMATS, formatBytes, type OutputFormat, type QueueItem } from "./types";
import "./App.css";

function newId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

function isImagePath(path: string): boolean {
  return /\.(png|jpe?g|webp|gif|bmp|tiff?|avif)$/i.test(path);
}

export default function App() {
  const [queue, setQueue] = useState<QueueItem[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [format, setFormat] = useState<OutputFormat>("webp");
  const [quality, setQuality] = useState(90);
  const [lossless, setLossless] = useState(false);
  const [suffix, setSuffix] = useState("");
  const [outputDir, setOutputDir] = useState<string | null>(null);
  const [dragging, setDragging] = useState(false);
  const [converting, setConverting] = useState(false);
  const [progress, setProgress] = useState({ done: 0, total: 0 });
  const [statusMsg, setStatusMsg] = useState<string | null>(null);

  const formatMeta = FORMATS.find((f) => f.id === format)!;
  const lossyNote = !lossless && (format === "jpeg" || format === "webp" || format === "avif" || format === "gif");

  const selected = useMemo(
    () => queue.find((q) => q.id === selectedId) ?? queue[0] ?? null,
    [queue, selectedId],
  );

  const enrichPaths = useCallback(async (paths: string[]) => {
    const unique = paths.filter(isImagePath);
    if (!unique.length) return;

    const starters: QueueItem[] = unique.map((path) => ({
      id: newId(),
      path,
      status: "pending",
    }));

    setQueue((prev) => {
      const existing = new Set(prev.map((p) => p.path.toLowerCase()));
      const fresh = starters.filter((s) => !existing.has(s.path.toLowerCase()));
      return [...prev, ...fresh];
    });

    for (const item of starters) {
      try {
        const [info, previewUrl] = await Promise.all([
          probeImage(item.path),
          previewImage(item.path, 420),
        ]);
        setQueue((prev) =>
          prev.map((q) =>
            q.path === item.path
              ? { ...q, info, previewUrl, status: "ready" }
              : q,
          ),
        );
        setSelectedId((cur) => cur ?? item.id);
      } catch (e) {
        const message = e instanceof Error ? e.message : String(e);
        setQueue((prev) =>
          prev.map((q) =>
            q.path === item.path
              ? { ...q, status: "error", error: message }
              : q,
          ),
        );
      }
    }
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        unlisten = await getCurrentWebview().onDragDropEvent((event) => {
          if (event.payload.type === "over") setDragging(true);
          if (event.payload.type === "leave" || event.payload.type === "drop") {
            setDragging(false);
          }
          if (event.payload.type === "drop") {
            void enrichPaths(event.payload.paths);
          }
        });
      } catch {
        // Browser preview without Tauri runtime
      }
    })();
    return () => {
      unlisten?.();
    };
  }, [enrichPaths]);

  async function handleBrowse() {
    const paths = await pickImageFiles();
    await enrichPaths(paths);
  }

  async function handlePickOutput() {
    const dir = await pickOutputFolder();
    if (dir) setOutputDir(dir);
  }

  function removeItem(id: string) {
    setQueue((prev) => prev.filter((q) => q.id !== id));
    setSelectedId((cur) => (cur === id ? null : cur));
  }

  function clearQueue() {
    setQueue([]);
    setSelectedId(null);
    setStatusMsg(null);
  }

  async function handleConvert() {
    if (!queue.length || !outputDir || converting) return;
    const ready = queue.filter((q) => q.status === "ready" || q.status === "done" || q.status === "error");
    if (!ready.length) return;

    setConverting(true);
    setProgress({ done: 0, total: ready.length });
    setStatusMsg(null);
    setQueue((prev) =>
      prev.map((q) =>
        ready.some((r) => r.id === q.id)
          ? { ...q, status: "converting", error: undefined, result: undefined }
          : q,
      ),
    );

    try {
      const result = await convertBatch(
        ready.map((q) => q.path),
        outputDir,
        {
          format,
          quality,
          lossless: lossless && formatMeta.lossless,
          background: [255, 255, 255],
          suffix: suffix.trim() || undefined,
        },
      );

      setQueue((prev) =>
        prev.map((q) => {
          const ok = result.successes.find((s) => s.sourcePath === q.path);
          if (ok) return { ...q, status: "done", result: ok };
          const fail = result.failures.find((f) => f.sourcePath === q.path);
          if (fail) return { ...q, status: "error", error: fail.error };
          return q;
        }),
      );
      setProgress({ done: ready.length, total: ready.length });
      setStatusMsg(
        `Converted ${result.successes.length} of ${ready.length}` +
          (result.failures.length ? ` · ${result.failures.length} failed` : ""),
      );
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setStatusMsg(message);
      setQueue((prev) =>
        prev.map((q) =>
          q.status === "converting" ? { ...q, status: "error", error: message } : q,
        ),
      );
    } finally {
      setConverting(false);
    }
  }

  const hasFiles = queue.length > 0;

  return (
    <div className={`app ${dragging ? "is-dragging" : ""}`}>
      <div className="bg-glow" aria-hidden />
      <div className="bg-grid" aria-hidden />

      <header className="topbar">
        <div className="brand-block">
          <p className="brand">Zayan Image Magic</p>
          <p className="tagline">Convert any image. Locally.</p>
        </div>
        <p className="maker">Made by Syed Faraz Ahmad</p>
      </header>

      <main className={`stage ${hasFiles ? "has-files" : ""}`}>
        {!hasFiles ? (
          <section
            className={`dropzone ${dragging ? "active" : ""}`}
            onClick={() => void handleBrowse()}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") void handleBrowse();
            }}
          >
            <div className="drop-orbit" aria-hidden />
            <div className="drop-core">
              <span className="drop-icon" aria-hidden />
              <h1>Drop images here</h1>
              <p>PNG, JPEG, WebP, AVIF, GIF, BMP, TIFF — all on your machine.</p>
              <button
                type="button"
                className="btn primary"
                onClick={(e) => {
                  e.stopPropagation();
                  void handleBrowse();
                }}
              >
                Browse files
              </button>
            </div>
          </section>
        ) : (
          <section className="workspace">
            <aside className="queue panel-enter">
              <div className="queue-head">
                <h2>{queue.length} file{queue.length === 1 ? "" : "s"}</h2>
                <div className="queue-actions">
                  <button type="button" className="btn ghost" onClick={() => void handleBrowse()}>
                    Add
                  </button>
                  <button type="button" className="btn ghost" onClick={clearQueue}>
                    Clear
                  </button>
                </div>
              </div>
              <ul className="queue-list">
                {queue.map((item) => (
                  <li key={item.id}>
                    <button
                      type="button"
                      className={`queue-item ${selected?.id === item.id ? "selected" : ""} status-${item.status}`}
                      onClick={() => setSelectedId(item.id)}
                    >
                      <span className="thumb">
                        {item.previewUrl ? (
                          <img src={item.previewUrl} alt="" />
                        ) : (
                          <span className="thumb-fallback" />
                        )}
                      </span>
                      <span className="meta">
                        <span className="name">{item.info?.fileName ?? item.path.split(/[/\\]/).pop()}</span>
                        <span className="sub">
                          {item.info
                            ? `${item.info.format} · ${item.info.width}×${item.info.height} · ${formatBytes(item.info.sizeBytes)}`
                            : item.status === "error"
                              ? item.error
                              : "Reading…"}
                        </span>
                        {item.result && (
                          <span className="sub ok">
                            → {formatBytes(item.result.outputBytes)} {item.result.format}
                          </span>
                        )}
                      </span>
                      <span
                        className="remove"
                        role="button"
                        tabIndex={0}
                        onClick={(e) => {
                          e.stopPropagation();
                          removeItem(item.id);
                        }}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") {
                            e.stopPropagation();
                            removeItem(item.id);
                          }
                        }}
                      >
                        ×
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            </aside>

            <div className="preview panel-enter">
              {selected?.previewUrl ? (
                <img src={selected.previewUrl} alt={selected.info?.fileName ?? "Preview"} />
              ) : (
                <div className="preview-empty">Select a file</div>
              )}
            </div>

            <div className="controls panel-enter">
              <h2>Convert to</h2>
              <div className="format-grid">
                {FORMATS.map((f) => (
                  <button
                    key={f.id}
                    type="button"
                    className={`format-chip ${format === f.id ? "active" : ""}`}
                    onClick={() => {
                      setFormat(f.id);
                      if (!f.lossless) setLossless(false);
                    }}
                  >
                    {f.label}
                  </button>
                ))}
              </div>

              <label className="field">
                <span>Quality {quality}</span>
                <input
                  type="range"
                  min={1}
                  max={100}
                  value={quality}
                  disabled={lossless && formatMeta.lossless}
                  onChange={(e) => setQuality(Number(e.target.value))}
                />
              </label>

              <label className={`toggle ${!formatMeta.lossless ? "disabled" : ""}`}>
                <input
                  type="checkbox"
                  checked={lossless && formatMeta.lossless}
                  disabled={!formatMeta.lossless}
                  onChange={(e) => setLossless(e.target.checked)}
                />
                <span>Lossless {formatMeta.lossless ? "" : "(n/a for this format)"}</span>
              </label>

              {lossyNote && (
                <p className="hint">
                  Lossy codecs cannot be bit-identical. High quality defaults keep visual fidelity.
                </p>
              )}

              <label className="field">
                <span>Filename suffix</span>
                <input
                  type="text"
                  placeholder="e.g. -converted"
                  value={suffix}
                  onChange={(e) => setSuffix(e.target.value)}
                />
              </label>

              <div className="output-row">
                <button type="button" className="btn ghost" onClick={() => void handlePickOutput()}>
                  Output folder
                </button>
                <p className="output-path">{outputDir ?? "Choose where files are saved"}</p>
              </div>

              <button
                type="button"
                className="btn primary convert"
                disabled={!outputDir || converting || !queue.some((q) => q.status !== "pending")}
                onClick={() => void handleConvert()}
              >
                {converting
                  ? `Converting ${progress.done}/${progress.total}…`
                  : `Convert ${queue.length} to ${format.toUpperCase()}`}
              </button>

              {converting && (
                <div className="progress-track" aria-hidden>
                  <div
                    className="progress-bar"
                    style={{
                      width: `${progress.total ? (progress.done / progress.total) * 100 : 12}%`,
                    }}
                  />
                </div>
              )}

              {statusMsg && <p className="status">{statusMsg}</p>}
            </div>
          </section>
        )}
      </main>

      <footer className="foot">
        <span>100% local · no uploads</span>
        <span>Made by Syed Faraz Ahmad</span>
      </footer>
    </div>
  );
}
