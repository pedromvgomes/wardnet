import type { WardnetClient } from "../client.js";
import type {
  InstallUpdateRequest,
  InstallUpdateResponse,
  RollbackResponse,
  UpdateCheckResponse,
  UpdateConfigRequest,
  UpdateConfigResponse,
  UpdateHistoryResponse,
  UpdateStatusResponse,
} from "../types/update.js";

/**
 * Auto-update service — status, manual check, install, rollback, config.
 *
 * All methods require admin authentication. The background runner on the
 * daemon side performs its own periodic checks; this service is the surface
 * for manual admin actions (`/update/check`, `/update/install`, etc.) and
 * for the Settings UI to read state.
 */
export class UpdateService {
  constructor(private readonly client: WardnetClient) {}

  /** Current update subsystem snapshot (admin only). */
  async status(): Promise<UpdateStatusResponse> {
    return this.client.request<UpdateStatusResponse>("/update/status");
  }

  /** Force a manifest refresh against the active channel (admin only). */
  async check(): Promise<UpdateCheckResponse> {
    return this.client.request<UpdateCheckResponse>("/update/check", { method: "POST" });
  }

  /**
   * Start an install. If `version` is omitted, installs the latest known
   * release on the active channel. Idempotent — calling twice while an
   * install is already in flight returns the same handle.
   */
  async install(body: InstallUpdateRequest = {}): Promise<InstallUpdateResponse> {
    return this.client.request<InstallUpdateResponse>("/update/install", {
      method: "POST",
      body: JSON.stringify(body),
    });
  }

  /** Swap back to `<live>.old` (admin only). Fails if no rollback is staged. */
  async rollback(): Promise<RollbackResponse> {
    return this.client.request<RollbackResponse>("/update/rollback", { method: "POST" });
  }

  /** Toggle auto-update / switch channel (admin only). */
  async updateConfig(body: UpdateConfigRequest): Promise<UpdateConfigResponse> {
    return this.client.request<UpdateConfigResponse>("/update/config", {
      method: "PUT",
      body: JSON.stringify(body),
    });
  }

  /** Recent install history entries (admin only). */
  async history(limit = 20): Promise<UpdateHistoryResponse> {
    // The SDK deliberately ships without DOM types, so URLSearchParams
    // isn't in scope; interpolating the integer limit directly is safe
    // (it's never user-controlled input).
    return this.client.request<UpdateHistoryResponse>(
      `/update/history?limit=${encodeURIComponent(limit)}`,
    );
  }
}
