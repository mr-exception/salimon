import { execFile } from "node:child_process";
import { promisify } from "node:util";
const execute = promisify(execFile);
export async function command(file: string, args: string[], timeout = 30000) {
  try {
    const { stdout } = await execute(file, args, { timeout, maxBuffer: 4 * 1024 * 1024, windowsHide: true });
    return stdout.trim();
  } catch (error) {
    // Do not expose command args or stderr: either may contain credentials.
    const e = error as NodeJS.ErrnoException;
    throw Error(`${file} operation failed (${e.code || "timeout or nonzero exit"})`);
  }
}
