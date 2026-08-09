export type TxOutcomeCategory =
  | "Unchanged"
  | "ComputeUnitsChanged"
  | "BehaviorChanged"
  | "ErrorChanged"
  | "NewFailure"
  | "NewSuccess";

export interface CounterStateSnapshot {
  is_initialized: boolean;
  authority: string;
  value: number;
}

export interface TxExecutionResult {
  label: string;
  success: boolean;
  error: string | null;
  logs: string[];
  compute_units_consumed: number;
  counter_state: CounterStateSnapshot | null;
}

export interface TxComparison {
  label: string;
  description: string;
  category: TxOutcomeCategory;
  old: TxExecutionResult;
  new: TxExecutionResult;
  compute_units_delta: number;
  notes: string;
}

export interface RunSummary {
  total: number;
  unchanged: number;
  compute_units_changed: number;
  behavior_changed: number;
  error_changed: number;
  new_failures: number;
  new_successes: number;
  safety_score: number;
}

export interface Report {
  old_program: string;
  new_program: string;
  summary: RunSummary;
  comparisons: TxComparison[];
}

export class PreflightApiError extends Error {}

async function unwrap<T>(response: Response): Promise<T> {
  if (!response.ok) {
    let message = `request failed with status ${response.status}`;
    try {
      const body = (await response.json()) as { error?: string };
      if (body.error) message = body.error;
    } catch {
      // response wasn't JSON; keep the generic message
    }
    throw new PreflightApiError(message);
  }
  return response.json() as Promise<T>;
}

/** Uploads two `.so` files and replays the bundled transaction fixture against both. */
export async function runPreflight(oldFile: File, newFile: File): Promise<Report> {
  const formData = new FormData();
  formData.append("old", oldFile);
  formData.append("new", newFile);

  const response = await fetch("/api/run", {
    method: "POST",
    body: formData,
  });
  return unwrap<Report>(response);
}

/** Runs the same pipeline against the bundled example counter program, no upload needed. */
export async function runDemo(): Promise<Report> {
  const response = await fetch("/api/demo", { method: "POST" });
  return unwrap<Report>(response);
}
