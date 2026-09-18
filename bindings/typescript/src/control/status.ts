import { z } from 'zod';

export const controlStatusSchema = z
	.object({
		mode: z.string(),
		quality_enabled: z.boolean(),
		exploration_enabled: z.boolean(),
		rtt_delta_ms: z.number().int().nonnegative(),
	})
	.passthrough()
	.readonly();

export type GetStatusResult = z.output<typeof controlStatusSchema>;
