// Sender CLI args, validation, and process helpers for srtla-send-rs.
// Export names MUST match @ceralive/srtla's ./sender subpath exactly (B2 lock)
// so the CeraUI migration is a mechanical import-source swap. Unlike the srtla
// bindings (node:child_process), this layer is Bun-native (Bun.spawn / Bun.which).
import { existsSync, statSync } from 'node:fs';
import { join } from 'node:path';

import { z } from 'zod';

const DEFAULT_BINARY = 'srtla_send';
const DEFAULT_SYSTEM_PATH = '/usr/bin/srtla_send';

export const srtlaSendOptionsSchema = z.object({
	listenPort: z
		.number()
		.int()
		.min(1)
		.max(65535)
		.default(5000)
		.describe('Local UDP port the sender listens on for SRT packets (positional arg 1).'),
	srtlaHost: z
		.string()
		.min(1)
		.describe('Hostname or IP of the SRTLA receiver / srtla_rec (positional arg 2).'),
	srtlaPort: z
		.number()
		.int()
		.min(1)
		.max(65535)
		.default(5001)
		.describe('UDP port of the SRTLA receiver (positional arg 3).'),
	ipsFile: z
		.string()
		.min(1)
		.default('/tmp/srtla_ips')
		.describe('Path to the newline-separated local source-IP (uplink) list (positional arg 4).'),
	verbose: z.boolean().optional(),
	statsFile: z.string().min(1).optional(),
	statsFileInterval: z.number().int().min(1).optional(),
	execPath: z.string().optional(),
});

export type SrtlaSendOptionsInput = z.input<typeof srtlaSendOptionsSchema>;
export type SrtlaSendOptions = z.output<typeof srtlaSendOptionsSchema>;

// Positional order is the load-bearing parity contract (B2 lock): the four
// positionals MUST emit as <listen_port> <srtla_host> <srtla_port> <ips_file>,
// then optional flags. CeraUI's migration depends on this exact vector.
export function buildSrtlaSendArgs(input: SrtlaSendOptionsInput): Array<string> {
	const options = srtlaSendOptionsSchema.parse(input);
	const args: Array<string> = [
		String(options.listenPort),
		options.srtlaHost,
		String(options.srtlaPort),
		options.ipsFile,
	];
	if (options.verbose) {
		args.push('--verbose');
	}
	if (options.statsFile) {
		args.push('--stats-file', options.statsFile);
	}
	if (options.statsFileInterval !== undefined) {
		args.push('--stats-file-interval', String(options.statsFileInterval));
	}
	return args;
}

export function getSrtlaSendExec(execPath?: string): string {
	if (execPath) {
		const stat = existsSync(execPath) ? statSync(execPath) : undefined;
		if (stat?.isFile()) {
			return execPath;
		}
		if (stat?.isDirectory()) {
			return join(execPath, DEFAULT_BINARY);
		}
		return execPath.endsWith(DEFAULT_BINARY) ? execPath : join(execPath, DEFAULT_BINARY);
	}

	const onPath = Bun.which(DEFAULT_BINARY);
	if (onPath) {
		return onPath;
	}
	if (existsSync(DEFAULT_SYSTEM_PATH)) {
		return DEFAULT_SYSTEM_PATH;
	}
	return DEFAULT_BINARY;
}

/**
 * Validate `options`, resolve the `srtla_send` binary, and spawn it with the
 * parity-locked positional argument vector.
 *
 * Throws a `ZodError` if `options` fails validation (e.g. an out-of-range port)
 * before any process is spawned. The binary is resolved via
 * {@link getSrtlaSendExec} (`options.execPath` → `PATH` → `/usr/bin/srtla_send`).
 *
 * @param options Sender options; the four positionals plus optional control-plane flags.
 * @returns The spawned `Bun.Subprocess`. The caller owns its lifecycle.
 */
export function spawnSrtlaSend(options: SrtlaSendOptionsInput): Bun.Subprocess {
	const parsed = srtlaSendOptionsSchema.parse(options);
	const exec = getSrtlaSendExec(parsed.execPath);
	const args = buildSrtlaSendArgs(parsed);
	return Bun.spawn([exec, ...args]);
}

/**
 * Signal every running `srtla_send` to reload its IP list (Unix `SIGHUP`).
 *
 * Mirrors the sender's live IP-list reload contract: surviving uplinks keep their
 * sockets, the pool is rebuilt in ips-file order. `killall` exits non-zero when no
 * process matches; that is treated as an acceptable no-op, so this never throws on
 * "nothing running".
 */
export function sendSrtlaSendHup(): void {
	// killall exits non-zero when no process matches; that is an acceptable no-op.
	Bun.spawnSync(['killall', '-HUP', DEFAULT_BINARY]);
}

export function isSrtlaSendRunning(): boolean {
	const result = Bun.spawnSync(['pgrep', '-x', DEFAULT_BINARY]);
	return result.exitCode === 0;
}
