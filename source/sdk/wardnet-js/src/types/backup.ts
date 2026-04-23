// Backup / restore types. Mirror the Rust definitions in
// `source/daemon/crates/wardnet-common/src/backup.rs` and `.../api.rs`.

/** Metadata describing a bundle. Serialised as `manifest.json` inside the tar. */
export interface BundleManifest {
  wardnet_version: string;
  schema_version: number;
  created_at: string;
  host_id: string;
  bundle_format_version: number;
  key_count: number;
}

/**
 * Phase of an in-flight restore. The tagged enum mirrors the Rust serde
 * shape — unlike `InstallPhase`, this one is not wrapped in an object
 * with a `phase` discriminator; it's a flat string except for the two
 * variants that carry data.
 */
export type RestorePhase =
  | "idle"
  | "validating"
  | "stopping_runners"
  | "backing_up"
  | "extracting"
  | "migrating"
  | "restarting_runners"
  | "applied"
  | { failed: { reason: string } };

/** Coarse subsystem status. Tagged union on `state`. */
export type BackupStatus =
  | { state: "idle" }
  | { state: "exporting" }
  | { state: "importing"; phase: RestorePhase }
  | { state: "failed"; reason: string };

/** What a retained snapshot is a snapshot of. */
export type SnapshotKind = "database" | "config" | "keys";

export interface LocalSnapshot {
  path: string;
  kind: SnapshotKind;
  created_at: string;
  size_bytes: number;
}

export interface BackupStatusResponse {
  status: BackupStatus;
}

export interface ExportBackupRequest {
  passphrase: string;
}

export interface RestorePreviewResponse {
  manifest: BundleManifest;
  compatible: boolean;
  incompatibility_reason?: string;
  files_to_replace: string[];
  preview_token: string;
}

export interface ApplyImportRequest {
  preview_token: string;
}

export interface ApplyImportResponse {
  manifest: BundleManifest;
  snapshots: LocalSnapshot[];
}

export interface ListSnapshotsResponse {
  snapshots: LocalSnapshot[];
}
