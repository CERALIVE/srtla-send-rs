import { afterEach, describe, expect, spyOn, test } from 'bun:test';

import { readTelemetry, SENDER_TELEMETRY_STALE_MS, watchTelemetry } from './index.js';

const created: Array<string> = [];

function tmpPath(): string {
	const p = `/tmp/srtla-send-tel-watch-test-${Date.now()}-${Math.random().toString(36).slice(2)}.json`;
	created.push(p);
	return p;
}

async function writeSnapshot(content: string): Promise<string> {
	const p = tmpPath();
	await Bun.write(p, content);
	return p;
}

afterEach(async () => {
	for (const p of created.splice(0)) {
		const deletion = Bun.file(p).delete?.();
		if (deletion !== undefined) {
			await deletion.catch(() => undefined);
		}
	}
});

function freshSnapshot(): string {
	return JSON.stringify({
		schema_version: 1,
		last_updated_ms: Date.now(),
		connections: [
			{
				conn_id: '0',
				rtt_ms: 42,
				nak_count: 3,
				weight_percent: 85,
				window: 8192,
				in_flight: 100,
				bitrate_bps: 2500000,
			},
		],
	});
}

const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));

describe('watchTelemetry', () => {
	test('reports stale=true for an absent file', async () => {
		const p = `/tmp/srtla-send-tel-watch-absent-${Date.now()}-${Math.random().toString(36).slice(2)}.json`;
		const updates: Array<{ data: unknown; stale: boolean }> = [];
		const handle = watchTelemetry(
			p,
			(u) => {
				updates.push(u);
			},
			{ intervalMs: 20 },
		);
		await sleep(50);
		handle.stop();
		expect(updates.length).toBeGreaterThanOrEqual(1);
		expect(updates.every((u) => u.data === null && u.stale)).toBe(true);
	});

	test('reports stale=false for a fresh snapshot, stale=true past the threshold', async () => {
		const FIXED = 1_800_000_000_000;
		const nowSpy = spyOn(Date, 'now').mockReturnValue(FIXED);
		try {
			const fresh = await writeSnapshot(
				JSON.stringify({ schema_version: 1, last_updated_ms: FIXED, connections: [] }),
			);
			const freshUpdates: Array<{ stale: boolean }> = [];
			const h1 = watchTelemetry(
				fresh,
				(u) => {
					freshUpdates.push(u);
				},
				{ intervalMs: 1000 },
			);
			await sleep(20);
			h1.stop();
			expect(freshUpdates[0]?.stale).toBe(false);

			const stale = await writeSnapshot(
				JSON.stringify({
					schema_version: 1,
					last_updated_ms: FIXED - SENDER_TELEMETRY_STALE_MS - 1,
					connections: [],
				}),
			);
			const staleUpdates: Array<{ data: unknown; stale: boolean }> = [];
			const h2 = watchTelemetry(
				stale,
				(u) => {
					staleUpdates.push(u);
				},
				{ intervalMs: 1000 },
			);
			await sleep(20);
			h2.stop();
			expect(staleUpdates[0]?.data).not.toBeNull();
			expect(staleUpdates[0]?.stale).toBe(true);
		} finally {
			nowSpy.mockRestore();
		}
	});

	test('stop() halts further callbacks', async () => {
		const p = await writeSnapshot(freshSnapshot());
		let calls = 0;
		const handle = watchTelemetry(
			p,
			() => {
				calls++;
			},
			{ intervalMs: 20 },
		);
		await sleep(50);
		handle.stop();
		const afterStop = calls;
		await sleep(80);
		expect(calls).toBe(afterStop);
	});
});

describe('watch states (T21)', () => {
	test('watch_state_fresh', async () => {
		const FIXED = 1_800_000_000_000;
		const nowSpy = spyOn(Date, 'now').mockReturnValue(FIXED);
		try {
			const p = await writeSnapshot(
				JSON.stringify({
					schema_version: 1,
					last_updated_ms: FIXED,
					connections: [
						{
							conn_id: '0',
							rtt_ms: 42,
							nak_count: 3,
							weight_percent: 85,
							window: 8192,
							in_flight: 100,
							bitrate_bps: 2500000,
						},
					],
				}),
			);
			const updates: Array<{ data: unknown; stale: boolean }> = [];
			const handle = watchTelemetry(
				p,
				(u) => {
					updates.push(u);
				},
				{ intervalMs: 1000 },
			);
			await sleep(20);
			handle.stop();
			expect(updates[0]?.data).not.toBeNull();
			expect(updates[0]?.stale).toBe(false);
		} finally {
			nowSpy.mockRestore();
		}
	});

	test('watch_state_stale', async () => {
		const FIXED = 1_800_000_000_000;
		const nowSpy = spyOn(Date, 'now').mockReturnValue(FIXED);
		try {
			const p = await writeSnapshot(
				JSON.stringify({
					schema_version: 1,
					last_updated_ms: FIXED - SENDER_TELEMETRY_STALE_MS - 1,
					connections: [
						{
							conn_id: '0',
							rtt_ms: 42,
							nak_count: 3,
							weight_percent: 85,
							window: 8192,
							in_flight: 100,
							bitrate_bps: 2500000,
						},
					],
				}),
			);
			const updates: Array<{ data: unknown; stale: boolean }> = [];
			const handle = watchTelemetry(
				p,
				(u) => {
					updates.push(u);
				},
				{ intervalMs: 1000 },
			);
			await sleep(20);
			handle.stop();
			expect(updates[0]?.data).not.toBeNull();
			expect(updates[0]?.stale).toBe(true);
		} finally {
			nowSpy.mockRestore();
		}
	});

	test('watch_state_null', async () => {
		const p = `/tmp/srtla-send-tel-watch-null-${Date.now()}-${Math.random().toString(36).slice(2)}.json`;
		expect(await readTelemetry(p)).toBeNull();

		const updates: Array<{ data: unknown; stale: boolean }> = [];
		const handle = watchTelemetry(
			p,
			(u) => {
				updates.push(u);
			},
			{ intervalMs: 20 },
		);
		await sleep(50);
		handle.stop();
		expect(updates.length).toBeGreaterThanOrEqual(1);
		expect(updates.every((u) => u.data === null && u.stale)).toBe(true);
	});
});
