import { invoke } from "@tauri-apps/api/core";

export function listPayloadPartitions(payloadPath) {
  return invoke("list_payload_partitions", { payloadPath });
}

export function extractPayloadPartitions(payloadPath, partitions, outputDir) {
  return invoke("extract_payload_partitions", { payloadPath, partitions, outputDir });
}
