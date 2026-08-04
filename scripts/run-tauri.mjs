import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const nasmDir = path.join(__dirname, "..", "tools", "nasm");
process.env.PATH = `${nasmDir}${path.delimiter}${process.env.PATH ?? ""}`;
process.env.CARGO_HTTP_CHECK_REVOKE = "false";

const args = process.argv.slice(2);
const child = spawn("tauri", args, {
  stdio: "inherit",
  shell: true,
  env: process.env,
});

child.on("exit", (code) => process.exit(code ?? 0));
