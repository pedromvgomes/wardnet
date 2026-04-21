import type { InstallPhase, UpdateChannel, UpdateStatus } from "@wardnet/js";
import { Button } from "@/components/core/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/core/ui/card";
import { Switch } from "@/components/core/ui/switch";
import { Label } from "@/components/core/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/core/ui/select";

/**
 * Whether a given install phase represents work in progress — i.e. a check/
 * download/verify/stage/swap is running server-side and the user should not
 * be able to kick off a second install on top of it.
 */
function isPhaseActive(phase: InstallPhase): boolean {
  switch (phase.phase) {
    case "checking":
    case "downloading":
    case "verifying":
    case "staging":
    case "swapping":
    case "restart_pending":
      return true;
    default:
      return false;
  }
}

function describePhase(phase: InstallPhase): string {
  switch (phase.phase) {
    case "downloading":
      return phase.total
        ? `Downloading (${phase.bytes.toLocaleString()} / ${phase.total.toLocaleString()} bytes)`
        : "Downloading";
    // Failed phase renders its reason in the alert banner; the phase cell
    // only needs to flag the state, not restate the error.
    case "failed":
      return "Failed";
    default:
      return phase.phase;
  }
}

interface Props {
  status: UpdateStatus | null;
  isLoading: boolean;
  isChecking: boolean;
  isInstalling: boolean;
  isRollingBack: boolean;
  onCheck: () => void;
  onInstall: () => void;
  onRollback: () => void;
  onToggleAutoUpdate: (enabled: boolean) => void;
  onChangeChannel: (channel: UpdateChannel) => void;
}

/**
 * Pure-presentation card showing version / channel / install state and
 * the admin action buttons. All data + callbacks flow from props; the card
 * itself does no API calls (that is the Settings page's job via hooks).
 */
export function UpdateCard({
  status,
  isLoading,
  isChecking,
  isInstalling,
  isRollingBack,
  onCheck,
  onInstall,
  onRollback,
  onToggleAutoUpdate,
  onChangeChannel,
}: Props) {
  // The install button must consider *both* the in-flight mutation and the
  // server-reported phase — without the latter, clicking Install would kick
  // off a second mutation after the first returns while the daemon is still
  // downloading in the background.
  const phaseActive = status ? isPhaseActive(status.install_phase) : false;
  const installButtonLabel = !status
    ? "Install"
    : isInstalling || phaseActive
      ? describePhase(status.install_phase) + "..."
      : status.update_available
        ? `Install v${status.latest_version}`
        : "Up to date";

  return (
    <Card>
      <CardHeader>
        <CardTitle>Auto-update</CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {isLoading || !status ? (
          <p className="text-sm text-muted-foreground">Loading...</p>
        ) : (
          <>
            {/* Channel row — the admin's primary choice sits first on its own
                row so the rest of the card (state + actions) reads as
                "consequences of this choice". */}
            <div className="flex items-center gap-3">
              <Label htmlFor="update-channel">Channel</Label>
              <Select
                value={status.channel}
                onValueChange={(v) => onChangeChannel(v as UpdateChannel)}
                disabled={phaseActive}
              >
                <SelectTrigger id="update-channel" className="w-[160px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="stable">Stable</SelectItem>
                  <SelectItem value="beta">Beta</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <dl className="grid grid-cols-2 gap-x-8 gap-y-3 text-sm sm:grid-cols-3">
              <div>
                <dt className="text-muted-foreground">Current version</dt>
                <dd className="font-medium">{status.current_version}</dd>
              </div>
              <div>
                <dt className="text-muted-foreground">Latest</dt>
                <dd className="font-medium">{status.latest_version ?? "unknown"}</dd>
              </div>
              <div>
                <dt className="text-muted-foreground">Last checked</dt>
                <dd className="font-medium">
                  {status.last_check_at ? new Date(status.last_check_at).toLocaleString() : "never"}
                </dd>
              </div>
              <div>
                <dt className="text-muted-foreground">Current phase</dt>
                <dd className="font-medium">{describePhase(status.install_phase)}</dd>
              </div>
              {status.pending_version && (
                <div>
                  <dt className="text-muted-foreground">Pending</dt>
                  <dd className="font-medium">{status.pending_version}</dd>
                </div>
              )}
            </dl>

            {status.install_phase.phase === "failed" && (
              <div
                role="alert"
                className="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive"
              >
                <div className="font-medium">Last install failed</div>
                <div className="mt-0.5 break-words">{status.install_phase.reason}</div>
              </div>
            )}

            <div className="flex items-center gap-3">
              <Switch
                id="auto-update-toggle"
                checked={status.auto_update_enabled}
                onCheckedChange={onToggleAutoUpdate}
              />
              <Label htmlFor="auto-update-toggle">Automatically install when available</Label>
            </div>

            <div className="flex flex-wrap gap-2">
              <Button variant="outline" onClick={onCheck} disabled={isChecking || phaseActive}>
                {isChecking ? "Checking..." : "Check for updates"}
              </Button>
              <Button
                onClick={onInstall}
                disabled={!status.update_available || isInstalling || phaseActive}
              >
                {installButtonLabel}
              </Button>
              {status.rollback_available && (
                <Button
                  variant="destructive"
                  onClick={onRollback}
                  disabled={isRollingBack || phaseActive}
                >
                  {isRollingBack ? "Rolling back..." : "Rollback to previous"}
                </Button>
              )}
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
