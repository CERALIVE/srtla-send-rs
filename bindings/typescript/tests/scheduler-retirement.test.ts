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
