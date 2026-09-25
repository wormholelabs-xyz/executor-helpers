#!/usr/bin/env bun
// Fetch core bridge state for the surfpool end-to-end tests.
//
// Usage: bun scripts/fetch-fixtures.ts <mainnet|testnet>
//
// Small accounts (bridge config, guardian set, program account) and the fixture
// VAA go to programs/post-vaa-relay/tests/fixtures/<net>/ (committed). The core
// bridge programdata account goes to target/fixtures/<net>/ (ignored, fetched on
// demand). All RPC calls go through the internal proxy with the required Origin
// header. No dependencies beyond the Bun runtime.

import { createHash } from "node:crypto";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ORIGIN = "https://portalbridge.com";
const PROGRAMDATA_HEADER_LEN = 45;
const FETCH_TIMEOUT_MS = 60_000;

type Network = "mainnet" | "testnet";

interface NetworkConfig {
  rpc: string;
  program: string;
  programdata: string;
  bridge: string;
  /** Guardian set PDA per index. Extend when the guardian set rotates. */
  guardianSets: Record<number, string>;
  scan: string;
  /** Fixed VAA id, or an emitter to take the newest VAA from. */
  vaa: { id: string } | { chain: number; emitter: string };
}

const NETWORKS: Record<Network, NetworkConfig> = {
  mainnet: {
    rpc: "https://rpc.labsapis.com/mainnet/solana",
    program: "worm2ZoG2kUd4vFXhvjh93UUH596ayRfgQ2MgjNMTth",
    programdata: "DUw5YjHpxeTVP5UZu39eoScMWDN5QnNnSrrBj68pFLHt",
    bridge: "2yVjuQwpsvdsrywzsJJVs9Ueh4zayyo5DYJbBNc3DDpn",
    guardianSets: { 7: "6YLGQQEweF82hbPSWCSeJqifWyT8Pm4QXa3mWSLwjYSh" },
    scan: "https://api.wormholescan.io",
    vaa: { id: "2/0000000000000000000000003ee18b2214aff97000d974cf647e7c347e8fa585/689930" },
  },
  // Wormhole "testnet" is the Solana devnet cluster.
  testnet: {
    rpc: "https://rpc.labsapis.com/testnet/solana",
    program: "3u8hJUVTA4jH1wYAyUur7FFZVQ8H635K3tSHHF4ssjQ5",
    programdata: "7bu9ccL3uu9xNCLE4ZuUX43sg7HUfort8YyLxcq6G9cQ",
    bridge: "6bi4JGDoRwUs9TYBuvoA7dUVyikTJDrJsJU1ew6KVLiu",
    guardianSets: { 0: "dxZtypiKT5D9LYzdPxjvSZER9MgYfeRVU5qpMTMTRs4" },
    scan: "https://api.testnet.wormholescan.io",
    // Token bridge emitter on Ethereum Sepolia (chain 10002).
    vaa: { chain: 10002, emitter: "000000000000000000000000db5492265f6038831e89f495670ff909ade94bd9" },
  },
};

interface AccountFixture {
  pubkey: string;
  slot: number;
  lamports: number;
  owner: string;
  executable: boolean;
  data_base64: string;
}

async function fetchJson(url: string, init: RequestInit): Promise<unknown> {
  const response = await fetch(url, { ...init, signal: AbortSignal.timeout(FETCH_TIMEOUT_MS) });
  if (!response.ok) {
    throw new Error(`${url}: HTTP ${response.status}`);
  }
  return response.json();
}

async function fetchAccount(rpc: string, pubkey: string): Promise<AccountFixture> {
  const body = {
    jsonrpc: "2.0",
    id: 1,
    method: "getAccountInfo",
    params: [pubkey, { encoding: "base64", commitment: "finalized" }],
  };
  const result = (await fetchJson(rpc, {
    method: "POST",
    headers: { "Content-Type": "application/json", Origin: ORIGIN },
    body: JSON.stringify(body),
  })) as {
    error?: unknown;
    result?: {
      context: { slot: number };
      value: { lamports: number; owner: string; executable: boolean; data: [string, string] } | null;
    };
  };
  if (result.error !== undefined) {
    throw new Error(`rpc error for ${pubkey}: ${JSON.stringify(result.error)}`);
  }
  const value = result.result?.value;
  if (value === null || value === undefined) {
    throw new Error(`account not found: ${pubkey}`);
  }
  if (value.data[1] !== "base64") {
    throw new Error(`unexpected encoding for ${pubkey}: ${value.data[1]}`);
  }
  return {
    pubkey,
    slot: result.result!.context.slot,
    lamports: value.lamports,
    owner: value.owner,
    executable: value.executable,
    data_base64: value.data[0],
  };
}

function writeJson(path: string, value: unknown): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, JSON.stringify(value, null, 2) + "\n");
}

async function main(): Promise<void> {
  const net = process.argv[2] as Network | undefined;
  if (net === undefined || !(net in NETWORKS)) {
    console.error("usage: bun scripts/fetch-fixtures.ts <mainnet|testnet>");
    process.exit(2);
  }
  const config = NETWORKS[net];
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const small = join(root, "programs/post-vaa-relay/tests/fixtures", net);
  const large = join(root, "target/fixtures", net);
  const log = (message: string) => console.log(`[${net}] ${message}`);

  log(`bridge config ${config.bridge}`);
  const bridge = await fetchAccount(config.rpc, config.bridge);
  writeJson(join(small, "bridge.json"), bridge);
  const guardianSetIndex = Buffer.from(bridge.data_base64, "base64").readUInt32LE(0);
  const guardianSetPubkey = config.guardianSets[guardianSetIndex];
  if (guardianSetPubkey === undefined) {
    throw new Error(`guardian set index ${guardianSetIndex} on ${net} is not in this script; add its PDA`);
  }
  log(`guardian set ${guardianSetIndex} ${guardianSetPubkey}`);
  writeJson(join(small, `guardian_set_${guardianSetIndex}.json`), await fetchAccount(config.rpc, guardianSetPubkey));

  log(`core bridge program ${config.program}`);
  writeJson(join(small, "core_bridge_program.json"), await fetchAccount(config.rpc, config.program));
  log(`core bridge programdata ${config.programdata} (large, into target/)`);
  const programdata = await fetchAccount(config.rpc, config.programdata);
  writeJson(join(large, "core_bridge_programdata.json"), programdata);
  const elf = Buffer.from(programdata.data_base64, "base64").subarray(PROGRAMDATA_HEADER_LEN);
  log(`ELF bytes = ${elf.length}, sha256 = ${createHash("sha256").update(elf).digest("hex")}`);

  let vaaId: string;
  if ("id" in config.vaa) {
    vaaId = config.vaa.id;
  } else {
    const newest = (await fetchJson(
      `${config.scan}/api/v1/vaas/${config.vaa.chain}/${config.vaa.emitter}?pageSize=1&sortOrder=DESC`,
      {},
    )) as { data: { id: string }[] };
    if (newest.data.length !== 1) {
      throw new Error(`no VAA found for emitter ${config.vaa.emitter}`);
    }
    vaaId = newest.data[0].id;
  }
  log(`VAA ${vaaId}`);
  const vaaResponse = (await fetchJson(`${config.scan}/api/v1/vaas/${vaaId}`, {})) as { data: { vaa: string } };
  const vaa = Buffer.from(vaaResponse.data.vaa, "base64");
  const fixture = {
    id: vaaId,
    guardian_set_index: vaa.readUInt32BE(1),
    signatures: vaa[5],
    vaa_base64: vaaResponse.data.vaa,
  };
  log(`VAA guardianSetIndex=${fixture.guardian_set_index} signatures=${fixture.signatures} bytes=${vaa.length}`);
  writeJson(join(small, "vaa.json"), fixture);
  log("done");
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
