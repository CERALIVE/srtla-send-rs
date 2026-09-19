import { z } from 'zod';

/**
 * Bond-wide receiver observation from the SRT handshake response (HSRSP) that
 * passed through the sender. Every key is omitted when unknown and the object is
 * `{}` before any handshake has been seen. Policy must read an absent
 * `nak_report` as NAK-on (`value ?? true`) while still displaying it as unknown.
 */
export const controlReceiverSchema = z
	.object({
		nak_report: z.boolean().optional(),
		srt_version: z.string().optional(),
		rexmit_flag: z.boolean().optional(),
	})
	.passthrough();

export type ControlReceiver = z.output<typeof controlReceiverSchema>;

/**
 * One `get-status.links[]` entry, in telemetry (`conn_id`) order. `iface`,
 * `link_id`, `health` and `priority` are omitted when they do not apply.
 * `rexmit_forwarded` counts SRT DATA forwarded on this link with the
 * retransmission bit set; it is a diagnostic only and never drives scheduling.
 * Binaries older than 4.0.0 do not emit it, so it stays optional here.
 */
export const controlLinkSchema = z
	.object({
		conn_id: z.string(),
		iface: z.string().optional(),
		link_id: z.string().optional(),
		health: z.string().optional(),
		priority: z.number().optional(),
		rexmit_forwarded: z.number().int().nonnegative().optional(),
	})
	.passthrough();

export type ControlLink = z.output<typeof controlLinkSchema>;

export const controlStatusSchema = z
	.object({
		mode: z.string(),
		quality_enabled: z.boolean(),
		exploration_enabled: z.boolean(),
		rtt_delta_ms: z.number().int().nonnegative(),
		negotiated_latency_ms: z.number().int().positive().optional(),
		receiver: controlReceiverSchema.optional(),
		links: z.array(controlLinkSchema).optional(),
	})
	.passthrough()
	.readonly();

export type GetStatusResult = z.output<typeof controlStatusSchema>;
