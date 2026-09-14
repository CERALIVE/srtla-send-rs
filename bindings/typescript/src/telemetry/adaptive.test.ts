import { expect, test } from 'bun:test';

import { connectionTelemetrySchema } from './index.js';

const legacy = {
	conn_id: '0',
	rtt_ms: 1,
	nak_count: 0,
	weight_percent: 100,
	window: 20000,
	in_flight: 0,
	bitrate_bps: 0,
};

test.each(['healthy', 'degraded', 'stalled', 'rejoining', 'down'])(
	'preserves %s with either priority boundary',
	(health) => {
		// Given each valid health and both bounds, When parsed, Then preserve both fields.
		for (const priority of [-0.2, 0, 0.2]) {
			const parsed = connectionTelemetrySchema.parse({ ...legacy, health, priority });
			expect(parsed.health).toBe(health);
			expect(parsed.priority).toBe(priority);
		}
	},
);

test.each([
	{ health: 'unknown' },
	{ health: null },
	{ health: 1 },
	{ priority: -0.21 },
	{ priority: 0.21 },
	{ priority: '0.2' },
	{ priority: null },
	{ priority: Number.NaN },
	{ priority: Number.POSITIVE_INFINITY },
])('rejects malformed scheduler observations %j', (fields) => {
	// Given malformed optional fields, When parsed, Then reject rather than strip them.
	expect(connectionTelemetrySchema.safeParse({ ...legacy, ...fields }).success).toBe(false);
});
