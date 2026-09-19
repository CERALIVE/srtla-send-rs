import { expect, test } from 'bun:test';
import { controlStatusSchema } from '../src/control/index.js';
import { srtlaSendOptionsSchema } from '../src/sender/index.js';

test.each(['classic', 'rtt-threshold', 'edpf', 'adaptive'])(
	'rejects retired spawn mode %s',
	(mode) => {
		// Given an untrusted caller requesting a deleted mode.
		const input = { srtlaHost: 'receiver', mode };
		// When parsing options at the spawn boundary.
		const result = srtlaSendOptionsSchema.safeParse(input);
		// Then it cannot reach the binary's startup arguments.
		expect(result.success).toBe(false);
	},
);

test.each(['classic', 'enhanced', 'edpf', 'future-scheduler'])(
	'preserves observed control mode %s without applying the spawn union',
	(mode) => {
		const result = controlStatusSchema.parse({
			mode,
			quality_enabled: true,
			exploration_enabled: false,
			rtt_delta_ms: 30,
		});
		expect(result.mode).toBe(mode);
	},
);

const baseStatus = {
	mode: 'enhanced',
	quality_enabled: true,
	exploration_enabled: false,
	rtt_delta_ms: 30,
};

test('types a 4.0.0 get-status with receiver, links[].rexmit_forwarded and latency', () => {
	// Given the shape the 4.0.0 binary returns after an HSRSP has passed through.
	const result = controlStatusSchema.parse({
		...baseStatus,
		negotiated_latency_ms: 2000,
		receiver: { nak_report: false, srt_version: '1.5.7', rexmit_flag: true },
		links: [
			{ conn_id: '0', iface: 'wwan0', link_id: 'modem-a', health: 'healthy', rexmit_forwarded: 12 },
			{ conn_id: '1', rexmit_forwarded: 0 },
		],
	});
	// Then every additive field is typed and preserved.
	expect(result.receiver?.nak_report).toBe(false);
	expect(result.receiver?.srt_version).toBe('1.5.7');
	expect(result.negotiated_latency_ms).toBe(2000);
	expect(result.links?.[0]?.rexmit_forwarded).toBe(12);
	expect(result.links?.[1]?.link_id).toBeUndefined();
});

test('reads an unknown receiver as absent, never as NAK-off', () => {
	// Given a status captured before any HSRSP (empty receiver object).
	const result = controlStatusSchema.parse({ ...baseStatus, receiver: {} });
	// Then the field is absent and the fail-safe policy read is NAK-on.
	expect(result.receiver?.nak_report).toBeUndefined();
	expect(result.receiver?.nak_report ?? true).toBe(true);
});

test('accepts a 3.3.0 get-status that has no receiver or links', () => {
	// Given the pre-4.0.0 shape.
	const result = controlStatusSchema.parse(baseStatus);
	// Then nothing is materialized for the additive fields.
	expect(result.receiver).toBeUndefined();
	expect(result.links).toBeUndefined();
});

test('rejects a negative rexmit_forwarded but passes unknown link keys through', () => {
	expect(
		controlStatusSchema.safeParse({
			...baseStatus,
			links: [{ conn_id: '0', rexmit_forwarded: -1 }],
		}).success,
	).toBe(false);
	const result = controlStatusSchema.parse({
		...baseStatus,
		links: [{ conn_id: '0', rexmit_forwarded: 1, future_key: 'kept' }],
	});
	const first: Record<string, unknown> | undefined = result.links?.[0];
	expect(first?.future_key).toBe('kept');
});
