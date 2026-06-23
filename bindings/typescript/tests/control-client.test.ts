import { afterEach, describe, expect, test } from 'bun:test';
import {
	createControlClient,
	type HelloResult,
	supportsStatsSubscription,
} from '../src/control/index.js';
import { buildSrtlaSendArgs, controlSocketPath } from '../src/sender/index.js';
import type { Telemetry } from '../src/telemetry/index.js';

// A fake JSON-RPC sender: a Unix-socket listener that replies to `hello` and,
// on `subscribe-events`, pushes a single replayed "event" notification (the T4
// wire shape). `onLine` decides each reply so a test can shape the server's
// capabilities or payload per case.
interface FakeServer {
	socketPath: string;
	stop(): void;
}

function fullSnapshot(): Telemetry {
	return {
		schema_version: 1,
		last_updated_ms: 1749556546000,
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
	};
}

const servers: FakeServer[] = [];

function startFakeServer(
	onLine: (line: string, reply: (frame: string) => void) => void,
): FakeServer {
	const socketPath = `/tmp/srtla-send-control-test-${Date.now()}-${Math.random().toString(36).slice(2)}.sock`;
	const listener = Bun.listen({
		unix: socketPath,
		socket: {
			data(socket, chunk) {
				const reply = (frame: string): void => {
					socket.write(`${frame}\n`);
				};
				for (const line of new TextDecoder().decode(chunk).split('\n')) {
					if (line.length > 0) {
						onLine(line, reply);
					}
				}
			},
		},
	});
	const server: FakeServer = {
		socketPath,
		stop() {
			listener.stop(true);
		},
	};
	servers.push(server);
	return server;
}

function helloReplyWith(capabilities: string[]) {
	return (line: string, reply: (frame: string) => void): void => {
		const req = JSON.parse(line) as { method: string; id: number };
		if (req.method === 'hello') {
			reply(
				JSON.stringify({
					jsonrpc: '2.0',
					id: req.id,
					result: { schema_version: 1, engine: 'srtla_send', capabilities },
				}),
			);
		}
	};
}

afterEach(() => {
	for (const server of servers.splice(0)) {
		server.stop();
	}
});

describe('hello → capabilities', () => {
	test('hello_returns_capabilities advertises stats-subscription', async () => {
		const server = startFakeServer(
			helloReplyWith(['stats-subscription', 'set-mode', 'get-status']),
		);
		const client = await createControlClient({ socketPath: server.socketPath });
		expect(client).not.toBeNull();
		if (client === null) return;
		const hello: HelloResult = await client.hello();
		expect(hello.engine).toBe('srtla_send');
		expect(hello.schema_version).toBe(1);
		expect(supportsStatsSubscription(hello)).toBe(true);
		client.close();
	});
});

describe('subscribeStats → event payload', () => {
	test('subscribeStats_parses_event_payload yields a typed Telemetry object', async () => {
		const snapshot = fullSnapshot();
		const server = startFakeServer((line, reply) => {
			const req = JSON.parse(line) as { method: string };
			if (req.method === 'subscribe-events') {
				reply(JSON.stringify({ jsonrpc: '2.0', method: 'event', params: snapshot }));
			}
		});
		const client = await createControlClient({ socketPath: server.socketPath });
		expect(client).not.toBeNull();
		if (client === null) return;

		const received = await new Promise<Telemetry | null>((resolve) => {
			const stop = client.subscribeStats((value) => {
				stop();
				resolve(value);
			});
		});
		expect(received).not.toBeNull();
		expect(received).toEqual(snapshot);
	});
});

describe('supportsStatsSubscription gate', () => {
	test('supportsStatsSubscription_false_on_missing_capability', async () => {
		const server = startFakeServer(helloReplyWith(['set-mode', 'get-status']));
		const client = await createControlClient({ socketPath: server.socketPath });
		expect(client).not.toBeNull();
		if (client === null) return;
		const hello = await client.hello();
		expect(supportsStatsSubscription(hello)).toBe(false);
		client.close();
	});
});

describe('connection failure', () => {
	test('createControlClient_returns_null_on_connect_failure', async () => {
		const absent = `/tmp/srtla-send-control-absent-${Date.now()}-${Math.random().toString(36).slice(2)}.sock`;
		const client = await createControlClient({ socketPath: absent });
		expect(client).toBeNull();
	});
});

describe('sender arg + path helpers', () => {
	test('buildSrtlaSendArgs_emits_control_socket', () => {
		const args = buildSrtlaSendArgs({
			listenPort: 5000,
			srtlaHost: 'rec.example.com',
			srtlaPort: 5001,
			ipsFile: '/tmp/srtla_ips',
			controlSocket: '/tmp/test.sock',
		});
		const idx = args.indexOf('--control-socket');
		expect(idx).toBeGreaterThanOrEqual(0);
		expect(args[idx + 1]).toBe('/tmp/test.sock');
	});

	test('controlSocketPath_derives_from_port', () => {
		expect(controlSocketPath(5000)).toBe('/tmp/srtla-send-control-5000.sock');
	});
});
